import { Mic } from 'lucide-react';

export function Header() {
  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4">
      <div className="flex items-center gap-3">
        <div className="flex items-center justify-center w-8 h-8 bg-blue-100 rounded-lg">
          <Mic className="w-5 h-5 text-blue-600" />
        </div>
        <div>
          <h1 className="text-xl font-semibold text-gray-900">Meeting Summarizer</h1>
          <p className="text-sm text-gray-500">Record and summarize your meetings</p>
        </div>
      </div>
    </header>
  );
}