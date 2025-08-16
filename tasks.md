# Meeting Summarizer - Implementation Tasks

## 2日間AI集中実装計画

Meeting Summarizerアプリを2日間でAI中心の実装により完成させる詳細なタスクリストです。

## Project Overview

- **期間**: 2日間 (16時間集中実装)
- **実装方針**: AI Agent主導
- **技術スタック**: Tauri + React + TypeScript + Jotai + shadcn/ui
- **完成目標**: 音声録音→保存→一覧表示のMVP

## Day 1: 基盤構築 + コア機能実装 (8時間)

### Morning Session (4時間) - 基盤構築

#### Task 1.1: プロジェクト初期化 (30分)
```bash
# Tauriプロジェクト作成
npx create-tauri-app meeting-summarizer --template react-ts
cd meeting-summarizer

# 基本依存関係インストール
npm install jotai lucide-react class-variance-authority clsx tailwind-merge
npm install -D @tailwindcss/typography eslint-config-prettier prettier
```

**自動生成ファイル**:
- [ ] `tauri.conf.json` - macOS権限設定
- [ ] `tsconfig.json` - strict mode設定
- [ ] `tailwind.config.js` - shadcn設定
- [ ] `.prettierrc` - フォーマット設定

#### Task 1.2: shadcn/ui セットアップ (45分)
```bash
# shadcn/ui初期化
npx shadcn-ui@latest init

# 必要なコンポーネントインストール
npx shadcn-ui@latest add button
npx shadcn-ui@latest add card
npx shadcn-ui@latest add dialog
npx shadcn-ui@latest add input
npx shadcn-ui@latest add progress
npx shadcn-ui@latest add separator
npx shadcn-ui@latest add badge
```

**自動構成**:
- [ ] `components.json` 設定ファイル
- [ ] `src/lib/utils.ts` ユーティリティ
- [ ] `src/components/ui/` shadcnコンポーネント

#### Task 1.3: プロジェクト構造作成 (45分)
```
src/
├── components/
│   ├── ui/              # shadcn/ui components
│   ├── features/
│   │   ├── recording/
│   │   │   ├── RecordingButton.tsx
│   │   │   ├── AudioVisualizer.tsx
│   │   │   └── RecordingTimer.tsx
│   │   └── files/
│   │       ├── FileList.tsx
│   │       └── FileCard.tsx
│   └── layout/
│       ├── AppLayout.tsx
│       ├── Header.tsx
│       └── Sidebar.tsx
├── atoms/
│   ├── recording.ts
│   ├── files.ts
│   └── ui.ts
├── hooks/
│   ├── useRecording.ts
│   └── useFiles.ts
├── types/
│   ├── recording.ts
│   └── index.ts
└── lib/
    ├── utils.ts
    └── constants.ts
```

**自動作成ファイル**:
- [ ] TypeScript型定義ファイル群
- [ ] Jotai atoms基本構造
- [ ] フォルダ構造とindex.tsファイル

#### Task 1.4: Rust Backend構造作成 (90分)
```
src-tauri/
├── src/
│   ├── commands/
│   │   ├── recording.rs
│   │   ├── files.rs
│   │   └── mod.rs
│   ├── services/
│   │   ├── recording_service.rs
│   │   ├── file_service.rs
│   │   └── mod.rs
│   ├── models/
│   │   ├── recording.rs
│   │   └── mod.rs
│   ├── infrastructure/
│   │   ├── database.rs
│   │   ├── audio.rs
│   │   └── mod.rs
│   ├── errors/
│   │   └── mod.rs
│   └── lib.rs
├── Cargo.toml
└── migrations/
    └── 001_initial.sql
```

**Cargo.toml依存関係**:
```toml
[dependencies]
tauri = "1.5"
serde = "1.0"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
rusqlite = { version = "0.29", features = ["bundled"] }
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
```

**自動生成ファイル**:
- [ ] Tauri command基本構造
- [ ] SQLite初期化コード
- [ ] エラーハンドリング基盤

### Afternoon Session (4時間) - コア機能実装

#### Task 1.5: SQLiteデータベース実装 (60分)
```sql
-- migrations/001_initial.sql
CREATE TABLE recordings (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL DEFAULT 0,
    duration INTEGER,
    format TEXT NOT NULL DEFAULT 'wav',
    status TEXT NOT NULL DEFAULT 'recording',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_recordings_created_at ON recordings(created_at DESC);
CREATE INDEX idx_recordings_status ON recordings(status);
```

**自動実装**:
- [ ] データベース接続管理
- [ ] 基本CRUD操作
- [ ] マイグレーション実行機能

#### Task 1.6: 音声録音API実装 (90分)
```rust
// commands/recording.rs
#[tauri::command]
pub async fn start_recording() -> Result<String, String>;

#[tauri::command]
pub async fn stop_recording() -> Result<Recording, String>;

#[tauri::command]
pub async fn get_recordings() -> Result<Vec<Recording>, String>;
```

**実装内容**:
- [ ] macOS Core Audio統合
- [ ] 音声録音開始/停止
- [ ] WAVファイル保存
- [ ] メタデータ管理

#### Task 1.7: Jotai状態管理実装 (45分)
```typescript
// atoms/recording.ts
export const isRecordingAtom = atom(false);
export const recordingDurationAtom = atom(0);
export const currentRecordingAtom = atom<Recording | null>(null);

// atoms/files.ts
export const recordingsAtom = atom<Recording[]>([]);
export const selectedRecordingAtom = atom<Recording | null>(null);
```

**自動実装**:
- [ ] 基本atom定義
- [ ] derived atoms
- [ ] 非同期atom (async)

#### Task 1.8: React Hooks実装 (45分)
```typescript
// hooks/useRecording.ts
export const useRecording = () => {
  // Tauri commands integration
  // State management with Jotai
  // Error handling
};
```

**自動実装**:
- [ ] Tauriコマンド統合
- [ ] エラーハンドリング
- [ ] リアルタイム更新

## Day 2: UI統合 + 最適化 + 完成 (8時間)

### Morning Session (4時間) - UI実装

#### Task 2.1: 基本レイアウト実装 (60分)
```typescript
// components/layout/AppLayout.tsx
export const AppLayout = () => {
  return (
    <div className="h-screen flex flex-col">
      <Header />
      <div className="flex flex-1">
        <Sidebar />
        <main className="flex-1 p-6">
          {children}
        </main>
      </div>
    </div>
  );
};
```

**shadcn/ui活用**:
- [ ] Card component for layouts
- [ ] Separator for visual separation
- [ ] Badge for status display

#### Task 2.2: 録音機能UI実装 (90分)
```typescript
// components/features/recording/RecordingButton.tsx
export const RecordingButton = () => {
  const { isRecording, startRecording, stopRecording } = useRecording();
  
  return (
    <Button
      size="lg"
      variant={isRecording ? "destructive" : "default"}
      className="w-32 h-32 rounded-full"
      onClick={isRecording ? stopRecording : startRecording}
    >
      {isRecording ? <Square /> : <Mic />}
    </Button>
  );
};
```

**実装内容**:
- [ ] 録音ボタンコンポーネント
- [ ] 録音時間表示
- [ ] 音声レベル可視化（基本版）
- [ ] 録音状態表示

#### Task 2.3: ファイル管理UI実装 (90分)
```typescript
// components/features/files/FileList.tsx
export const FileList = () => {
  const [recordings] = useAtom(recordingsAtom);
  
  return (
    <div className="space-y-4">
      {recordings.map(recording => (
        <FileCard key={recording.id} recording={recording} />
      ))}
    </div>
  );
};
```

**実装内容**:
- [ ] ファイル一覧表示
- [ ] ファイル詳細カード
- [ ] 削除・再生ボタン
- [ ] 検索・フィルター（基本版）

### Afternoon Session (4時間) - 統合・最適化

#### Task 2.4: フロントエンド・バックエンド統合 (90分)
```typescript
// App.tsx
export const App = () => {
  const loadRecordings = useCallback(async () => {
    try {
      const recordings = await invoke<Recording[]>('get_recordings');
      setRecordings(recordings);
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }, []);

  useEffect(() => {
    loadRecordings();
  }, [loadRecordings]);

  return (
    <AppLayout>
      <RecordingSection />
      <FileListSection />
    </AppLayout>
  );
};
```

**統合内容**:
- [ ] Tauri commands呼び出し
- [ ] エラーハンドリング統合
- [ ] リアルタイム状態同期
- [ ] ローディング状態管理

#### Task 2.5: エラーハンドリング・UX改善 (60分)
```typescript
// エラートースト表示
// ローディングスピナー
// 空状態の表示
// 権限エラーハンドリング
```

**実装内容**:
- [ ] Toast notification システム
- [ ] ローディング状態表示
- [ ] エラー状態の適切な表示
- [ ] マイクアクセス権限チェック

#### Task 2.6: macOSビルド・動作確認 (60分)
```bash
# 開発ビルド確認
npm run tauri dev

# プロダクションビルド
npm run tauri build

# 署名・配布準備
codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" target/release/bundle/macos/Meeting\ Summarizer.app
```

**検証項目**:
- [ ] アプリ起動確認
- [ ] 音声録音機能動作
- [ ] ファイル保存・読み込み
- [ ] UI操作の応答性

#### Task 2.7: ドキュメント・最終調整 (30分)
```markdown
# README.md更新
# 使用方法ドキュメント
# トラブルシューティング
```

**最終確認**:
- [ ] 基本機能動作確認
- [ ] パフォーマンス確認
- [ ] エラーケース確認
- [ ] ドキュメント完成

## 完成基準 (MVP)

### 必須機能
- ✅ 音声録音開始・停止
- ✅ 録音ファイル自動保存
- ✅ 録音ファイル一覧表示
- ✅ ファイル削除機能
- ✅ 基本的なエラーハンドリング

### UI/UX要件
- ✅ 直感的な録音ボタン
- ✅ 録音時間表示
- ✅ ファイル一覧の見やすい表示
- ✅ 録音状態の明確な表示
- ✅ shadcn/uiによる統一感あるデザイン

### 技術要件
- ✅ macOSアプリとして動作
- ✅ TypeScript strict mode
- ✅ Clean Architecture構造
- ✅ Jotaiによる状態管理
- ✅ SQLiteデータ永続化

## AI実装支援スクリプト

### 自動セットアップスクリプト
```bash
#!/bin/bash
# setup.sh - プロジェクト自動セットアップ

echo "🚀 Meeting Summarizer セットアップ開始"

# Tauriプロジェクト作成
npx create-tauri-app meeting-summarizer --template react-ts
cd meeting-summarizer

# 依存関係インストール
npm install jotai lucide-react class-variance-authority clsx tailwind-merge
npm install -D @tailwindcss/typography eslint-config-prettier prettier

# shadcn/ui セットアップ
npx shadcn-ui@latest init --yes
npx shadcn-ui@latest add button card dialog input progress separator badge

echo "✅ セットアップ完了"
```

### 開発支援スクリプト
```bash
#!/bin/bash
# dev.sh - 開発開始スクリプト

# 開発サーバー起動
npm run tauri dev &

# TypeScript型チェック
npm run type-check --watch &

echo "🔥 開発環境起動完了"
```

## 品質チェックリスト

### Day 1 完了チェック
- [ ] プロジェクトが正常にビルドされる
- [ ] 基本的なUI表示が動作する
- [ ] Rustバックエンドが起動する
- [ ] SQLiteデータベースが初期化される
- [ ] 基本的な音声録音APIが動作する

### Day 2 完了チェック
- [ ] 音声録音・停止が完全に動作する
- [ ] ファイル保存・読み込みが動作する
- [ ] UI操作がスムーズに動作する
- [ ] エラーが適切にハンドリングされる
- [ ] macOSアプリとしてビルドできる

### 最終品質確認
- [ ] アプリ起動時間: 5秒以内
- [ ] 録音開始応答: 2秒以内
- [ ] ファイル一覧読み込み: 1秒以内
- [ ] メモリ使用量: 300MB以下
- [ ] TypeScriptエラー: 0件

## 次のPhase準備

MVP完成後、以下のPhaseに向けた準備：
- [ ] Whisper統合の設計確認
- [ ] LLM統合アーキテクチャ準備
- [ ] パフォーマンス最適化計画
- [ ] ユーザーフィードバック収集準備

---

## 実装開始準備完了 🎯

すべての設計・計画が完了しました。このタスクリストに基づいて2日間の集中実装を開始できます。

**準備状況**:
- ✅ Requirements定義完了
- ✅ Design仕様完了  
- ✅ Implementation計画完了
- ✅ shadcn/ui技術スタック確定
- ✅ 2日間集中スケジュール調整済み

次のアクション: `npm run tauri dev` でプロジェクト作成開始