# Meeting Summarizer - Requirements Specification

## Project Overview

**Meeting Summarizer** は、macOSデスクトップ向けの会議録音・書き起こし・要約アプリケーションです。完全ローカル処理により、プライバシーを保護しながら高品質な議事録を生成します。

## User Stories

### Core User Stories

1. **開発者として**、プロジェクトを迷いなく開始できるよう、標準化された構造を持ちたい
2. **開発者として**、macOSネイティブアプリとして高品質な体験を提供するため、適切なフレームワークを選択したい
3. **開発者として**、継続的な開発を支援するため、自動化されたビルド・テスト環境を構築したい
4. **開発者として**、将来的な機能拡張に対応するため、スケーラブルなアーキテクチャを準備したい

## EARS記法による要件定義

### 基本要件

```
WHEN 開発者がプロジェクトをクローンした際
GIVEN 必要な開発ツールがインストールされている場合
THEN システムは一発でローカル開発環境を構築する
AND 30秒以内にアプリケーションが起動可能な状態になる
```

```
WHILE 開発者がコードを編集している間
THE SYSTEM SHALL ホットリロード機能を提供する
AND TypeScript型チェックをリアルタイムで実行する
AND ESLint/Prettierによるコード品質チェックを自動実行する
```

```
WHERE macOSシステム要件において
IS 音声録音・ファイルアクセス・LLM実行が必要な場合
THE SYSTEM SHALL 適切な権限要求とエラーハンドリングを実装する
AND システムリソース使用量を最適化する
```

## Technical Stack

### Core Technologies
- **Framework**: Tauri
- **Frontend**: React + TypeScript
- **State Management**: Zustand
- **UI Components**: Tailwind CSS + Radix UI
- **Backend**: Rust
- **Database**: SQLite + Rusqlite

### Platform Requirements
- **Target OS**: macOS 12.0+
- **Architecture**: Intel + Apple Silicon
- **Permissions**: Microphone, File System

## Security Requirements

### Privacy Protection
- 音声データの完全ローカル処理
- 外部サーバーへのデータ送信なし
- 暗号化されたローカルストレージ

### System Security
- macOSサンドボックス対応
- 最小権限の原則
- コード署名とノータリゼーション

## Performance Requirements

### Response Time
- アプリ起動時間: 3秒以内
- 音声録音開始: 1秒以内
- UI応答性: 60fps維持
- ホットリロード: 2秒以内

### Resource Usage
- メモリ使用量: 200MB以下（アイドル時）
- バイナリサイズ: 50MB以下
- CPU使用率: 待機時5%以下

## Quality Requirements

### Code Quality
- TypeScript strict mode
- ESLint + Prettier自動適用
- 単体テストカバレッジ 80%+
- E2Eテスト自動化

### Development Experience
- ワンコマンドセットアップ
- 自動化されたCI/CD
- 開発用デバッグツール
- ホットリロード対応

## Compatibility Requirements

### macOS Versions
- macOS Monterey (12.0) +
- macOS Ventura (13.0) 対応
- macOS Sonoma (14.0) 最適化

### Hardware Support
- Intel Mac対応
- Apple Silicon M1/M2/M3対応
- 最小メモリ: 8GB
- 推奨メモリ: 16GB+

## Error Handling Requirements

### System Errors
- 音声デバイス不在時の適切な通知
- ディスク容量不足時の警告
- 権限不足時の案内表示

### Development Errors
- TypeScript型エラーの即座表示
- ビルドエラーの詳細表示
- テスト失敗時の問題箇所特定

## Acceptance Criteria

### Phase 1 Completion Criteria

#### Environment Setup
- [ ] `npm run dev` で開発サーバーが起動
- [ ] `npm run build` でmacOSアプリが生成
- [ ] TypeScript型チェックが動作
- [ ] ESLint/Prettierが自動実行

#### Project Structure
- [ ] 標準化されたディレクトリ構造
- [ ] 設定ファイルの適切な配置
- [ ] 依存関係の明確な管理

#### Development Tools
- [ ] ホットリロード機能
- [ ] デバッグ環境
- [ ] テスト実行環境
- [ ] CI/CD基盤

## Dependencies

### Development Dependencies
- Node.js 18+
- Rust 1.70+
- Tauri CLI
- Git

### External Systems
- macOS Core Audio (将来的)
- whisper.cpp (将来的)
- Ollama (将来的)

## Risks and Mitigation

### Technical Risks
- **Tauri学習コスト**: 段階的導入、ドキュメント整備
- **Rust開発速度**: TypeScript中心開発、最小限Rust利用
- **macOS権限**: 開発初期段階での検証

### Project Risks
- **複雑性増大**: 段階的開発、MVP重視
- **パフォーマンス**: 継続的測定、最適化

## Success Metrics

### Development Efficiency
- 新規開発者のセットアップ時間: 10分以内
- ビルド時間: 30秒以内
- テスト実行時間: 10秒以内

### Code Quality
- TypeScriptコンパイルエラー: 0
- ESLintエラー: 0
- テストカバレッジ: 80%+