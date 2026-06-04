import { describe, it, expect } from 'vitest';
import { buildColumns, makeTimestamp, buildHeaderMismatchError } from '../mappingHelpers';

// ── buildColumns ──────────────────────────────────────────────────────────────

describe('buildColumns', () => {
  it('MJSと他社で名称が完全一致するヘッダーを自動マッピングする', () => {
    const mjs = ['社員コード', '出勤日数', '欠勤日数'];
    const other = ['社員コード', '出勤日数', '残業時間'];
    const cols = buildColumns(mjs, other);

    expect(cols[0].otherHeader).toBe('社員コード'); // 一致
    expect(cols[1].otherHeader).toBe('出勤日数');   // 一致
    expect(cols[2].otherHeader).toBe('');            // 不一致 → 空
  });

  it('先頭列（インデックス0）は isCompare=false になる', () => {
    const cols = buildColumns(['社員コード', '出勤日数'], ['社員コード', '出勤日数']);
    expect(cols[0].isCompare).toBe(false);
    expect(cols[1].isCompare).toBe(true);
  });

  it('tolerance の初期値はすべて 0 になる', () => {
    const cols = buildColumns(['社員コード', '出勤日数'], ['社員コード', '出勤日数']);
    expect(cols[0].tolerance).toBe(0);
    expect(cols[1].tolerance).toBe(0);
  });

  it('他社ヘッダーが空の場合、すべて未マッピング（空文字）になる', () => {
    const cols = buildColumns(['社員コード', '出勤日数'], []);
    expect(cols[0].otherHeader).toBe('');
    expect(cols[1].otherHeader).toBe('');
  });

  it('MJSヘッダーが1列のみでも正常に動作する（その列がキー列）', () => {
    const cols = buildColumns(['社員コード'], ['社員コード']);
    expect(cols.length).toBe(1);
    expect(cols[0].isCompare).toBe(false);
  });

  it('MJSヘッダーが空のとき空配列を返す', () => {
    const cols = buildColumns([], []);
    expect(cols).toEqual([]);
  });

  it('大文字小文字の差異はマッピングされない（完全一致のみ）', () => {
    const cols = buildColumns(['EMP_CODE'], ['emp_code']);
    expect(cols[0].otherHeader).toBe('');
  });
});

// ── makeTimestamp ─────────────────────────────────────────────────────────────

describe('makeTimestamp', () => {
  it('yyyymmddhhmm 形式（12桁の数字）を返す', () => {
    const ts = makeTimestamp();
    expect(ts).toMatch(/^\d{12}$/);
  });

  it('現在年で始まる文字列を返す', () => {
    const ts = makeTimestamp();
    const year = new Date().getFullYear().toString();
    expect(ts.startsWith(year)).toBe(true);
  });

  it('月・日・時・分がゼロパディングされる', () => {
    // 12桁であれば各部分が正しくパディングされている
    const ts = makeTimestamp();
    expect(ts.length).toBe(12);
  });
});

// ── buildHeaderMismatchError ──────────────────────────────────────────────────

describe('buildHeaderMismatchError', () => {
  it('ヘッダーが完全一致する場合は null を返す', () => {
    const result = buildHeaderMismatchError(
      ['社員コード', '出勤日数'],
      ['社員コード', '出勤日数']
    );
    expect(result).toBeNull();
  });

  it('どちらも空の場合は null を返す', () => {
    expect(buildHeaderMismatchError([], [])).toBeNull();
  });

  it('設定ファイルにのみ存在するヘッダーをエラーメッセージに含める', () => {
    const result = buildHeaderMismatchError(
      ['社員コード', '旧列'],
      ['社員コード']
    );
    expect(result).not.toBeNull();
    expect(result).toContain('設定ファイルにのみ存在: 旧列');
  });

  it('CSVにのみ存在するヘッダーをエラーメッセージに含める', () => {
    const result = buildHeaderMismatchError(
      ['社員コード'],
      ['社員コード', '新列']
    );
    expect(result).not.toBeNull();
    expect(result).toContain('CSVにのみ存在: 新列');
  });

  it('双方向の差分を同時に検出する', () => {
    const result = buildHeaderMismatchError(
      ['社員コード', '旧列'],
      ['社員コード', '新列']
    );
    expect(result).toContain('設定ファイルにのみ存在: 旧列');
    expect(result).toContain('CSVにのみ存在: 新列');
  });

  it('複数の不一致ヘッダーをカンマ区切りで列挙する', () => {
    const result = buildHeaderMismatchError(
      ['社員コード', '旧列A', '旧列B'],
      ['社員コード']
    );
    expect(result).toContain('旧列A, 旧列B');
  });

  it('エラー時は先頭に案内メッセージを含める', () => {
    const result = buildHeaderMismatchError(['社員コード', '旧列'], ['社員コード']);
    expect(result).toContain('設定を適用できません');
  });
});
