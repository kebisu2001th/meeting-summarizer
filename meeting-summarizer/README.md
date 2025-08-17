# Meeting Summarizer

音声録音・書き起こし・要約機能を持つ macOS デスクトップアプリです。
完全オフラインで動作し、プライバシーを重視した設計になっています。

## 🎯 機能概要

- **🎙️ 音声録音**: ワンクリックでの録音開始・停止
- **📝 音声書き起こし**: OpenAI Whisper による高精度な書き起こし（完全ローカル実行）
- **🌐 オフライン動作**: インターネット接続不要、完全プライベート
- **⏱️ リアルタイムタイマー**: mm:ss 形式の録音時間表示
- **📁 ファイル管理**: 録音ファイルの一覧表示・削除
- **💾 データ永続化**: SQLite による録音メタデータ管理
- **🎨 モダン UI**: shadcn/ui による macOS ライクなクリーンデザイン
- **🔒 プライバシー保護**: 音声データは全てローカルに保存

## 🛠️ 技術スタック

### フロントエンド
- **React 18** + **TypeScript** (型安全な UI 開発)
- **Jotai** (効率的な状態管理)
- **shadcn/ui** + **Tailwind CSS** (モダンな UI コンポーネント)
- **Lucide React** (アイコンシステム)
- **Vite** (高速ビルドツール)

### バックエンド
- **Rust** + **Tauri** (パフォーマンスとセキュリティ)
- **SQLite** + **Rusqlite** (軽量データベース)
- **Tokio** (非同期処理)

## 📋 前提条件

### 一般ユーザー向け
- **macOS** 10.15 (Catalina) 以上
- **Python** 3.8 以上（システムに標準でインストール済み）
- 最低 2GB の空きディスク容量（Whisper モデル用）

### 開発者向け
- **Node.js** 20.14.0 以上
- **pnpm** 10.14.0 以上
- **Rust** 1.70 以上
- **Tauri CLI** 2.x
- **Python** 3.8 以上

### バージョン管理ツール（推奨）
- **asdf** または **mise** (.tool-versions 対応)

## 📱 アプリのインストール（一般ユーザー向け）

### インストール方法

現在、Meeting Summarizer は開発段階にあります。以下の手順でビルド済みアプリケーションを入手できます：

#### 1. 事前ビルド版のダウンロード
```bash
# リポジトリをクローン
git clone https://github.com/kebisu2001th/meeting-summarizer.git
cd meeting-summarizer/meeting-summarizer

# ビルド済みアプリケーションを生成
pnpm install
pnpm run tauri build
```

#### 2. アプリケーションのインストール
```bash
# ビルドされた .app ファイルを Applications フォルダに移動
cp -r src-tauri/target/release/bundle/macos/Meeting\ Summarizer.app /Applications/
```

### 初回起動時のセットアップ

1. **アプリケーション起動**: Applications フォルダから Meeting Summarizer を起動
2. **セキュリティ許可**: macOS が開発者の確認を求める場合、「システム環境設定 → セキュリティとプライバシー」で許可
3. **Whisper ライブラリ自動インストール**: 初回書き起こし時に自動でインストール（数分かかります）
4. **マイクロフォン許可**: 音声録音のためのマイクロフォンアクセスを許可

### システム要件
- **macOS**: 10.15 (Catalina) 以上
- **Python**: 3.8 以上（通常は macOS に標準インストール）
- **ディスク容量**: 最低 2GB（Whisper モデル用）
- **メモリ**: 最低 4GB RAM 推奨

### 使用方法
1. アプリを起動
2. 中央の録音ボタンをクリックして録音開始
3. 録音を停止すると自動で書き起こしが実行されます
4. 下部のリストで録音ファイルを管理できます

---

## 🚀 開発者向けセットアップ手順

### 1. リポジトリのクローン

```bash
git clone https://github.com/kebisu2001th/meeting-summarizer.git
cd meeting-summarizer/meeting-summarizer
```

### 2. 開発環境の準備

#### Option A: asdf/mise を使用する場合（推奨）

```bash
# asdf の場合
asdf install

# mise の場合
mise install
```

#### Option B: 手動インストール

**Node.js & pnpm のインストール:**
```bash
# Node.js 20.14.0 をインストール
# https://nodejs.org/ からダウンロード

# pnpm のインストール
npm install -g pnpm@10.14.0
```

**Rust のインストール:**
```bash
# Rust のインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# バージョン確認
rustc --version
cargo --version
```

**Tauri CLI のインストール:**
```bash
cargo install tauri-cli --version "^2.0"
```

### 3. 依存関係のインストール

```bash
# プロジェクトディレクトリに移動
cd meeting-summarizer

# フロントエンド依存関係のインストール
pnpm install
```

### 4. 開発サーバーの起動

```bash
# 開発環境での起動（ホットリロード対応）
pnpm run tauri dev
```

初回起動時は Rust のコンパイルが実行されるため、数分かかる場合があります。

## 📦 ビルド

### 開発ビルド

```bash
# フロントエンドのみビルド（型チェック込み）
pnpm run build

# フロントエンドビルド結果をプレビュー
pnpm run preview
```

### 本番ビルド

```bash
# Tauri アプリケーションのビルド（.app ファイル生成）
pnpm run tauri build
```

ビルド成果物は `src-tauri/target/release/bundle/` に生成されます。

## 🗂️ プロジェクト構造

```
meeting-summarizer/
├── src/                          # React フロントエンド
│   ├── components/
│   │   ├── features/             # 機能別コンポーネント
│   │   │   ├── RecordingControls.tsx
│   │   │   └── RecordingsList.tsx
│   │   ├── layout/               # レイアウトコンポーネント
│   │   └── ui/                   # shadcn/ui コンポーネント
│   ├── atoms/                    # Jotai 状態管理
│   │   └── recording.ts
│   ├── hooks/                    # カスタムフック
│   ├── services/                 # Tauri 統合
│   │   └── tauri.ts
│   └── types/                    # TypeScript 型定義
├── src-tauri/                    # Rust バックエンド
│   ├── src/
│   │   ├── commands/             # Tauri コマンド（IPC）
│   │   ├── services/             # ビジネスロジック
│   │   ├── database/             # SQLite 管理
│   │   ├── models/               # ドメインモデル
│   │   └── errors/               # エラーハンドリング
│   └── Cargo.toml
├── public/                       # 静的ファイル
├── .tool-versions               # バージョン固定ファイル
├── package.json                 # フロントエンド依存関係
├── tailwind.config.js           # Tailwind CSS 設定
└── vite.config.ts              # Vite 設定
```

## 🧪 テスト

```bash
# 型チェック
pnpm run build

# Rust テスト
cd src-tauri
cargo test
```

## 📱 使用方法

1. **アプリケーション起動**: `pnpm run tauri dev` でアプリが起動
2. **録音開始**: 中央の録音ボタンをクリック
3. **録音停止**: 再度ボタンをクリックして録音終了
4. **ファイル管理**: 下部リストで録音ファイルの確認・削除

## 🔧 開発時の注意点

### ホットリロード
- フロントエンドの変更は自動でリロードされます
- Rust コードの変更時は自動でリコンパイルされます

### デバッグ
- ブラウザの開発者ツールでフロントエンドをデバッグ
- Rust 側のログは Terminal に出力されます

### データ保存場所
録音ファイルとデータベースは以下に保存されます：
- **macOS**: `~/Library/Application Support/meeting-summarizer/`

## 🛠️ トラブルシューティング

### よくある問題

**1. `pnpm not found` エラー**
```bash
npm install -g pnpm@10.14.0
```

**2. Rust コンパイルエラー**
```bash
# Rust ツールチェーンの更新
rustup update
```

**3. Tauri CLI が見つからない**
```bash
cargo install tauri-cli --version "^2.0"
```

**4. 権限エラー（macOS）**
```bash
# アプリに必要な権限を手動で許可
# システム環境設定 → セキュリティとプライバシー
```

### ログの確認

開発時のログ出力：
```bash
# 詳細ログを有効にして起動
RUST_LOG=debug pnpm run tauri dev
```

## 🚀 次のステップ

現在の MVP から以下の機能拡張が予定されています：

### Phase 2: 音声処理実装 ✅
- [x] **ローカル Whisper** による音声書き起こし（Python 実装）
- [x] 自動 Whisper ライブラリインストール機能
- [x] モック音声録音実装（実際の macOS API は今後対応予定）
- [x] 音声ファイル形式最適化（WAV 形式対応）

### Phase 3: LLM 統合
- [ ] **Ollama** ローカル実行環境
- [ ] 議事録自動生成
- [ ] 要約品質向上

### Phase 4: UX 向上
- [ ] 音声レベル可視化
- [ ] ファイル再生機能
- [ ] 設定画面
- [ ] エクスポート機能

## 🏗️ アーキテクチャ

### Clean Architecture
- **UI Layer**: React コンポーネント
- **Application Layer**: Jotai atoms (状態管理)
- **Domain Layer**: Rust models
- **Infrastructure Layer**: SQLite, Tauri IPC

### 設計原則
- **単一責任の原則**: 各モジュールが明確な責任を持つ
- **依存性逆転**: 上位レイヤーが下位レイヤーに依存しない
- **テスト可能性**: モックとスタブによる単体テスト対応

## 📄 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。

## 🤝 コントリビューション

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feat-amazing-feature`)
3. 変更をコミット (`git commit -m 'feat: 素晴らしい機能を追加'`)
4. ブランチにプッシュ (`git push origin feat-amazing-feature`)
5. Pull Request を作成

## 📞 サポート

問題が発生した場合は、以下をお試しください：

1. [Issues](https://github.com/kebisu2001th/meeting-summarizer/issues) で既知の問題を確認
2. 新しい Issue を作成して問題を報告
3. 詳細なエラーメッセージとシステム情報を含めてください

## 推奨 IDE 設定

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

---

**Happy Coding! 🎉**
