import { Provider } from 'jotai';
import { Header } from './components/layout/Header';
import { RecordingControls } from './components/features/RecordingControls';
import { RecordingsList } from './components/features/RecordingsList';

function App() {
  return (
    <Provider>
      <div className="min-h-screen bg-gray-50">
        {/* ヘッダー */}
        <Header />
        
        {/* メインコンテンツ */}
        <main className="container mx-auto px-6 py-8 max-w-4xl">
          <div className="space-y-8">
            {/* 録音セクション */}
            <section className="flex justify-center">
              <RecordingControls />
            </section>
            
            {/* ファイル一覧セクション */}
            <section>
              <RecordingsList />
            </section>
          </div>
        </main>
      </div>
    </Provider>
  );
}

export default App;
