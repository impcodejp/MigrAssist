import type { ColumnMapping } from '../types';

/**
 * MJSヘッダーを元に突合設定列を初期化する。
 * 他社ヘッダーと名称が完全一致する場合は自動でマッピングする。
 */
export function buildColumns(mjsHeaders: string[], otherHeaders: string[]): ColumnMapping[] {
  return mjsHeaders.map((mjs, idx) => ({
    mjsHeader: mjs,
    otherHeader: otherHeaders.includes(mjs) ? mjs : '',
    isCompare: idx > 0, // 先頭列はキー列のため比較対象外
    tolerance: 0,
  }));
}

/** タイムスタンプ文字列を yyyymmddhhmm 形式で生成する。 */
export function makeTimestamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}${p(d.getHours())}${p(d.getMinutes())}`;
}

/**
 * 設定ファイルのヘッダーと現在のCSVヘッダーを比較し、不一致があればエラーメッセージを返す。
 * 一致している場合は null を返す。
 */
export function buildHeaderMismatchError(
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
