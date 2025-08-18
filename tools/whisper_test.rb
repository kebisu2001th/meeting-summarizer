#!/usr/bin/env ruby
# frozen_string_literal: true

# 音声書き起こしテスト用CLIツール
# Usage: ruby whisper_test.rb <audio_file_path> [language]

require 'json'
require 'open3'
require 'optparse'
require 'pathname'

class WhisperTester
  def initialize
    @model_size = 'small'  # 速度と精度のバランス
    @language = 'ja'
    @verbose = false
    @output_format = 'text'
    @whisper_params = {}
  end

  def run(args)
    parse_options(args)
    
    if args.empty?
      puts help_message
      exit 1
    end
    
    audio_file = args.first
    validate_audio_file(audio_file)
    
    puts "🎤 音声書き起こしテスト開始" if @verbose
    puts "📁 ファイル: #{audio_file}" if @verbose
    puts "🌐 言語: #{@language}" if @verbose
    puts "🧠 モデル: #{@model_size}" if @verbose
    
    result = transcribe_audio(audio_file)
    output_result(result)
  end

  private

  def parse_options(args)
    OptionParser.new do |opts|
      opts.banner = "Usage: #{$0} [options] <audio_file>"
      
      opts.on('-l', '--language LANG', 'Language code (default: ja)') do |lang|
        @language = lang
      end
      
      opts.on('-m', '--model MODEL', 'Whisper model size (tiny, base, small, medium, large)') do |model|
        @model_size = model
      end
      
      opts.on('-v', '--verbose', 'Verbose output') do
        @verbose = true
      end
      
      opts.on('-f', '--format FORMAT', 'Output format (text, json, srt)') do |format|
        @output_format = format
      end
      
      opts.on('-t', '--temperature TEMP', Float, 'Temperature for sampling (0.0-1.0)') do |temp|
        @whisper_params[:temperature] = temp
      end
      
      opts.on('-b', '--best-of N', Integer, 'Number of candidates to consider') do |n|
        @whisper_params[:best_of] = n
      end
      
      opts.on('-s', '--beam-size N', Integer, 'Beam size for search') do |n|
        @whisper_params[:beam_size] = n
      end
      
      opts.on('--no-preprocessing', 'Skip audio preprocessing') do
        @whisper_params[:no_preprocessing] = true
      end
      
      opts.on('-h', '--help', 'Show this help') do
        puts opts
        exit
      end
    end.parse!(args)
  end

  def validate_audio_file(file_path)
    path = Pathname.new(file_path)
    
    unless path.exist?
      puts "❌ エラー: ファイルが見つかりません: #{file_path}"
      exit 1
    end
    
    unless path.file?
      puts "❌ エラー: ディレクトリが指定されました: #{file_path}"
      exit 1
    end
    
    # 音声ファイル形式のチェック
    valid_extensions = %w[.wav .mp3 .m4a .flac .ogg .aac .mp4 .mov .avi]
    unless valid_extensions.include?(path.extname.downcase)
      puts "⚠️  警告: 対応していない可能性のあるファイル形式: #{path.extname}"
      puts "   対応形式: #{valid_extensions.join(', ')}"
    end
    
    # ファイルサイズチェック (500MB制限)
    file_size = path.size
    max_size = 500 * 1024 * 1024
    
    if file_size > max_size
      puts "❌ エラー: ファイルサイズが大きすぎます: #{format_file_size(file_size)}"
      puts "   最大サイズ: #{format_file_size(max_size)}"
      exit 1
    end
    
    puts "✅ ファイル検証完了: #{format_file_size(file_size)}" if @verbose
  end

  def transcribe_audio(audio_file)
    script = build_whisper_script(audio_file)
    
    puts "🐍 Pythonスクリプト実行中..." if @verbose
    
    start_time = Time.now
    stdout, stderr, status = Open3.capture3('python3', '-c', script)
    end_time = Time.now
    
    processing_time = ((end_time - start_time) * 1000).round
    
    if status.success?
      {
        text: stdout.strip,
        stderr: stderr,
        processing_time_ms: processing_time,
        success: true
      }
    else
      {
        error: stderr,
        stdout: stdout,
        processing_time_ms: processing_time,
        success: false
      }
    end
  end

  def build_whisper_script(audio_file)
    # Whisperパラメータの構築
    params = build_whisper_params
    preprocessing = @whisper_params[:no_preprocessing] ? false : true
    
    if preprocessing
      audio_processing = preprocessing_code
    else
      audio_processing = "    audio_data = audio_file"
    end

    script = <<~PYTHON
import whisper
import sys
import warnings
import os
import json
warnings.filterwarnings("ignore")

try:
    audio_file = '#{audio_file}'
    
    # ファイル存在確認
    if not os.path.exists(audio_file):
        print(f"Error: Audio file not found: {audio_file}", file=sys.stderr)
        sys.exit(1)
    
    file_size = os.path.getsize(audio_file)
    print(f"Loading model: #{@model_size}", file=sys.stderr)
    
    # GPU加速の設定（現在は互換性のため CPU のみ）
    model = whisper.load_model('#{@model_size}')
    
    print(f"Processing: {audio_file} ({file_size} bytes)", file=sys.stderr)
    
    # 音声前処理
#{audio_processing}
    
    # 書き起こし実行
    result = model.transcribe(
        audio_data,
        #{params}
    )
    
    # 結果の処理
    text = result.get('text', '').strip()
    
    # 日本語の後処理
    if '#{@language}' == 'ja':
        import re
        # プロンプトテキストの削除
        prompt_patterns = [
            '日本語の音声です：',
            '以下は日本語の音声です：',
            '日本語の音声です。',
            '以下は日本語の音声です。'
        ]
        
        for pattern in prompt_patterns:
            while pattern in text:
                text = text.replace(pattern, '', 1).strip()
        
        # テキスト整形
        text = re.sub(r'\\s+', ' ', text).strip()
        
        if not text.strip():
            text = "音声を認識できませんでした。"
    
    # 出力形式に応じて結果を出力
    if '#{@output_format}' == 'json':
        output = {
            'text': text,
            'language': result.get('language', '#{@language}'),
            'segments': result.get('segments', []),
            'duration': result.get('duration', 0),
            'model': '#{@model_size}',
            'preprocessing': #{preprocessing ? 'True' : 'False'}
        }
        print(json.dumps(output, ensure_ascii=False, indent=2))
    elif '#{@output_format}' == 'srt':
        segments = result.get('segments', [])
        for i, segment in enumerate(segments, 1):
            start = segment.get('start', 0)
            end = segment.get('end', 0)
            text_seg = segment.get('text', '').strip()
            
            start_time = format_time(start)
            end_time = format_time(end)
            
            print(f"{i}")
            print(f"{start_time} --> {end_time}")
            print(text_seg)
            print()
    else:
        print(text)
        
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    import traceback
    traceback.print_exc(file=sys.stderr)
    sys.exit(1)

def format_time(seconds):
    hours = int(seconds // 3600)
    minutes = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    millis = int((seconds % 1) * 1000)
    return f"{hours:02d}:{minutes:02d}:{secs:02d},{millis:03d}"
    PYTHON
    
    script
  end

  def preprocessing_code
    <<~PYTHON.strip
    try:
        import librosa
        import numpy as np
        
        print("Audio preprocessing with librosa...", file=sys.stderr)
        audio_data, sr = librosa.load(audio_file, sr=16000)
        
        # ボリューム正規化
        rms = np.sqrt(np.mean(audio_data**2))
        if rms > 0:
            target_rms = 0.1
            audio_data = audio_data * (target_rms / rms)
        
        # 無音部分の除去
        audio_data, _ = librosa.effects.trim(audio_data, top_db=30)
        print(f"Preprocessing completed: {len(audio_data)} samples", file=sys.stderr)
        
    except ImportError:
        print("librosa not available, using direct file processing", file=sys.stderr)
        audio_data = audio_file
    PYTHON
  end

  def build_whisper_params
    params = {
      :language => @language,
      :task => 'transcribe'
    }
    
    # 高速化デフォルトパラメータ
    if @language == 'ja'
      params.merge!({
        :temperature => 0.2,
        :best_of => 1,
        :beam_size => 1,
        :patience => 1.0,
        :length_penalty => 1.0,
        :suppress_tokens => [-1],
        :word_timestamps => false,
        :condition_on_previous_text => true
      })
    else
      params.merge!({
        :temperature => 0.2,
        :best_of => 1,
        :beam_size => 1
      })
    end
    
    # ユーザー指定パラメータで上書き
    params.merge!(@whisper_params.reject { |k, v| k == :no_preprocessing })
    
    # Python辞書形式に変換
    params.map do |key, value|
      case value
      when String
        "#{key}='#{value}'"
      when true, false
        "#{key}=#{value ? 'True' : 'False'}"
      when Array
        "#{key}=#{value}"
      else
        "#{key}=#{value}"
      end
    end.join(",\n            ")
  end

  def output_result(result)
    if result[:success]
      puts "✅ 書き起こし完了 (#{result[:processing_time_ms]}ms)" if @verbose
      puts result[:text] if @output_format == 'text'
      
      if @verbose && !result[:stderr].empty?
        puts "\n--- デバッグ情報 ---"
        puts result[:stderr]
      end
    else
      puts "❌ 書き起こし失敗"
      puts "エラー: #{result[:error]}"
      puts "stdout: #{result[:stdout]}" unless result[:stdout].empty?
      exit 1
    end
  end

  def format_file_size(bytes)
    units = ['B', 'KB', 'MB', 'GB']
    size = bytes.to_f
    unit_index = 0
    
    while size >= 1024 && unit_index < units.length - 1
      size /= 1024
      unit_index += 1
    end
    
    if unit_index == 0
      "#{size.to_i} #{units[unit_index]}"
    else
      "#{size.round(2)} #{units[unit_index]}"
    end
  end

  def help_message
    <<~HELP
      🎤 音声書き起こしテスト用CLIツール

      使用方法:
        #{$0} [オプション] <音声ファイル>

      例:
        #{$0} recording.wav
        #{$0} --language en --model large audio.mp3
        #{$0} --format json --verbose meeting.m4a
        #{$0} --format srt presentation.wav

      オプション:
        -l, --language LANG      言語コード (デフォルト: ja)
        -m, --model MODEL        Whisperモデル (tiny, base, small, medium, large)
        -f, --format FORMAT      出力形式 (text, json, srt)
        -v, --verbose            詳細出力
        -t, --temperature TEMP   サンプリング温度 (0.0-1.0)
        -b, --best-of N          候補数
        -s, --beam-size N        ビームサイズ
        --no-preprocessing       音声前処理をスキップ
        -h, --help               このヘルプを表示

      対応ファイル形式:
        .wav, .mp3, .m4a, .flac, .ogg, .aac, .mp4, .mov, .avi

      必要環境:
        - Python 3.8以上
        - openai-whisper
        - librosa (音声前処理用、オプション)
    HELP
  end
end

# メイン実行
if __FILE__ == $0
  WhisperTester.new.run(ARGV)
end