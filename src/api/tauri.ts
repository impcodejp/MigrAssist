import { invoke } from '@tauri-apps/api/core';
import { message, open, save } from '@tauri-apps/plugin-dialog';
import type { ComparisonConfig, ConfigData } from '../types';

export async function loadCsv(path: string, encoding: string): Promise<string[]> {
  return invoke<string[]>('load_csv', { path, encoding });
}

export async function executeComparison(config: ComparisonConfig): Promise<void> {
  await invoke('execute_comparison', { config });
}

export async function saveConfig(path: string, data: ConfigData): Promise<void> {
  return invoke('save_config', { path, data });
}

export async function loadConfig(path: string): Promise<ConfigData> {
  return invoke<ConfigData>('load_config', { path });
}

export async function pickCsvFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: 'CSVファイル', extensions: ['csv'] }],
  });
  return typeof result === 'string' ? result : null;
}

export async function pickJsonFileForOpen(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: '設定ファイル', extensions: ['json'] }],
  });
  return typeof result === 'string' ? result : null;
}

export async function pickJsonFileForSave(): Promise<string | null> {
  return save({
    filters: [{ name: '設定ファイル', extensions: ['json'] }],
    defaultPath: 'mapping-config.json',
  });
}

export async function pickDirectory(): Promise<string | null> {
  const result = await open({ directory: true });
  return typeof result === 'string' ? result : null;
}

export async function showCompletionDialog(): Promise<void> {
  await message('突合処理が完了しました。\n出力フォルダのファイルを確認してください。', {
    title: '完了',
    kind: 'info',
  });
}
