import { useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import * as api from '../api/tauri';
import type { ColumnMapping, Encoding, ProgressPayload } from '../types';

// ── State shape ───────────────────────────────────────────────────────────────

export interface AppState {
  mjsPath: string;
  mjsEncoding: Encoding;
  otherPath: string;
  otherEncoding: Encoding;
  mjsHeaders: string[] | null;
  otherHeaders: string[] | null;
  columns: ColumnMapping[];
  outputDir: string;
  isRunning: boolean;
  progress: ProgressPayload | null;
  error: string | null;
}

const initialState: AppState = {
  mjsPath: '',
  mjsEncoding: 'sjis',
  otherPath: '',
  otherEncoding: 'sjis',
  mjsHeaders: null,
  otherHeaders: null,
  columns: [],
  outputDir: '',
  isRunning: false,
  progress: null,
  error: null,
};

// ── Pure helpers ──────────────────────────────────────────────────────────────

/**
 * MJSヘッダーを元に突合設定列を初期化する。
 * 他社ヘッダーと名称が完全一致する場合は自動でマッピングする。
 */
function buildColumns(mjsHeaders: string[], otherHeaders: string[]): ColumnMapping[] {
  return mjsHeaders.map((mjs, idx) => ({
    mjsHeader: mjs,
    otherHeader: otherHeaders.includes(mjs) ? mjs : '',
    isCompare: idx > 0, // 先頭列はキー列のため比較対象外
    tolerance: 0,
  }));
}

/** タイムスタンプ文字列を yyyymmddhhmm 形式で生成する。 */
function makeTimestamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}${p(d.getHours())}${p(d.getMinutes())}`;
}

/**
 * 設定ファイルのヘッダーと現在のCSVヘッダーを比較し、不一致があればエラーメッセージを返す。
 * 一致している場合は null を返す。
 */
function buildHeaderMismatchError(
  configHeaders: string[],
  csvHeaders: string[]
): string | null {
  const onlyInConfig = configHeaders.filter((h) => !csvHeaders.includes(h));
  const onlyInCsv = csvHeaders.filter((h) => !configHeaders.includes(h));
  if (onlyInConfig.length === 0 && onlyInCsv.length === 0) return null;

  const lines = ['設定ファイルのヘッダーと取込済みCSVのヘッダーが一致しないため、設定を適用できません。'];
  if (onlyInConfig.length > 0) lines.push(`設定ファイルにのみ存在: ${onlyInConfig.join(', ')}`);
  if (onlyInCsv.length > 0) lines.push(`CSVにのみ存在: ${onlyInCsv.join(', ')}`);
  return lines.join('\n');
}

// ── Hook ──────────────────────────────────────────────────────────────────────

export function useAppState() {
  const [state, setState] = useState<AppState>(initialState);

  const setError = useCallback((error: string | null) => {
    setState((s) => ({ ...s, error }));
  }, []);

  const setMjsPath = useCallback((mjsPath: string) => {
    setState((s) => ({ ...s, mjsPath }));
  }, []);

  const setMjsEncoding = useCallback((mjsEncoding: Encoding) => {
    setState((s) => ({ ...s, mjsEncoding }));
  }, []);

  const setOtherPath = useCallback((otherPath: string) => {
    setState((s) => ({ ...s, otherPath }));
  }, []);

  const setOtherEncoding = useCallback((otherEncoding: Encoding) => {
    setState((s) => ({ ...s, otherEncoding }));
  }, []);

  const setOutputDir = useCallback((outputDir: string) => {
    setState((s) => ({ ...s, outputDir }));
  }, []);

  const updateColumn = useCallback(
    (idx: number, patch: Partial<ColumnMapping>) => {
      setState((s) => {
        const columns = s.columns.map((col, i) =>
          i === idx ? { ...col, ...patch } : col
        );
        return { ...s, columns };
      });
    },
    []
  );

  const loadFiles = useCallback(async () => {
    setState((s) => ({ ...s, error: null }));
    try {
      const [mjsHeaders, otherHeaders] = await Promise.all([
        api.loadCsv(state.mjsPath, state.mjsEncoding),
        api.loadCsv(state.otherPath, state.otherEncoding),
      ]);
      const columns = buildColumns(mjsHeaders, otherHeaders);
      setState((s) => ({ ...s, mjsHeaders, otherHeaders, columns }));
    } catch (e) {
      setState((s) => ({ ...s, error: String(e) }));
    }
  }, [state.mjsPath, state.mjsEncoding, state.otherPath, state.otherEncoding]);

  const execute = useCallback(async () => {
    setState((s) => ({ ...s, isRunning: true, progress: null, error: null }));

    const unlisten = await listen<ProgressPayload>(
      'comparison-progress',
      (event) => {
        setState((s) => ({ ...s, progress: event.payload }));
      }
    );

    try {
      await api.executeComparison({
        mjsPath: state.mjsPath,
        mjsEncoding: state.mjsEncoding,
        otherPath: state.otherPath,
        otherEncoding: state.otherEncoding,
        outputDir: state.outputDir,
        columns: state.columns,
        timestamp: makeTimestamp(),
      });
      await api.showCompletionDialog();
    } catch (e) {
      setState((s) => ({ ...s, error: String(e) }));
    } finally {
      unlisten();
      setState((s) => ({ ...s, isRunning: false }));
    }
  }, [
    state.mjsPath,
    state.mjsEncoding,
    state.otherPath,
    state.otherEncoding,
    state.outputDir,
    state.columns,
  ]);

  const saveConfig = useCallback(async () => {
    const path = await api.pickJsonFileForSave();
    if (!path) return;
    try {
      await api.saveConfig(path, { columns: state.columns });
    } catch (e) {
      setState((s) => ({ ...s, error: String(e) }));
    }
  }, [state.columns]);

  const loadConfig = useCallback(async () => {
    const path = await api.pickJsonFileForOpen();
    if (!path) return;
    try {
      const data = await api.loadConfig(path);
      const csvHeaders = state.mjsHeaders ?? [];
      const configHeaders = data.columns.map((c) => c.mjsHeader);

      const mismatchError = buildHeaderMismatchError(configHeaders, csvHeaders);
      if (mismatchError) {
        setState((s) => ({ ...s, error: mismatchError }));
        return;
      }

      setState((s) => ({ ...s, columns: data.columns }));
    } catch (e) {
      setState((s) => ({ ...s, error: String(e) }));
    }
  }, [state.mjsHeaders]);

  const pickMjsFile = useCallback(async () => {
    const path = await api.pickCsvFile();
    if (path) setMjsPath(path);
  }, [setMjsPath]);

  const pickOtherFile = useCallback(async () => {
    const path = await api.pickCsvFile();
    if (path) setOtherPath(path);
  }, [setOtherPath]);

  const pickOutputDir = useCallback(async () => {
    const path = await api.pickDirectory();
    if (path) setOutputDir(path);
  }, [setOutputDir]);

  return {
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
  };
}
