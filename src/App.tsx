import { useAppState } from './hooks/useAppState';
import { FileInputSection } from './components/FileInputSection';
import { MappingTable } from './components/MappingTable';
import { ProgressBar } from './components/ProgressBar';
import './App.css';

function App() {
  const {
    state,
    setMjsPath,
    setMjsEncoding,
    setOtherPath,
    setOtherEncoding,
    setOutputDir,
    updateColumn,
    loadFiles,
    execute,
    saveConfig,
    loadConfig,
    pickMjsFile,
    pickOtherFile,
    pickOutputDir,
    setError,
  } = useAppState();

  const filesLoaded = state.mjsHeaders !== null && state.otherHeaders !== null;

  return (
    <div className="app">
      {/* ── ファイル入力エリア ── */}
      <section className="card">
        <FileInputSection
          label="MJSシステム"
          path={state.mjsPath}
          encoding={state.mjsEncoding}
          onPathChange={setMjsPath}
          onEncodingChange={setMjsEncoding}
          onBrowse={pickMjsFile}
        />
        <FileInputSection
          label="他社システム"
          path={state.otherPath}
          encoding={state.otherEncoding}
          onPathChange={setOtherPath}
          onEncodingChange={setOtherEncoding}
          onBrowse={pickOtherFile}
        />
        <div className="action-row">
          <button
            className="btn btn-primary"
            onClick={loadFiles}
            disabled={!state.mjsPath || !state.otherPath}
          >
            ファイル取込
          </button>
        </div>
      </section>

      {/* ── 突合設定テーブル ── */}
      <section className="card card-table">
        <div className="table-toolbar">
          <button
            className="btn btn-secondary"
            onClick={saveConfig}
            disabled={!filesLoaded}
          >
            設定出力
          </button>
          <button
            className="btn btn-secondary"
            onClick={loadConfig}
            disabled={!filesLoaded}
          >
            設定取込
          </button>
        </div>
        <MappingTable
          columns={state.columns}
          otherHeaders={state.otherHeaders ?? []}
          onUpdate={updateColumn}
        />
      </section>

      {/* ── 出力フォルダ ── */}
      <section className="card">
        <div className="file-input-row">
          <span className="file-input-label">出力フォルダ</span>
          <div className="file-input-controls">
            <input
              type="text"
              className="file-path-input"
              value={state.outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
              placeholder="出力先フォルダを入力または参照ボタンで選択"
            />
            <button className="btn btn-secondary" onClick={pickOutputDir}>
              参照
            </button>
          </div>
        </div>
        <div className="action-row">
          <button
            className="btn btn-execute"
            onClick={execute}
            disabled={state.isRunning || !filesLoaded}
          >
            {state.isRunning ? '実行中...' : '実行'}
          </button>
        </div>
      </section>

      {/* ── 進捗バー ── */}
      {state.isRunning && state.progress && (
        <section className="card">
          <ProgressBar progress={state.progress} />
        </section>
      )}

      {/* ── エラー表示 ── */}
      {state.error && (
        <section className="card card-error">
          <div className="error-message">{state.error}</div>
          <button className="btn btn-secondary" onClick={() => setError(null)}>
            閉じる
          </button>
        </section>
      )}

    </div>
  );
}

export default App;
