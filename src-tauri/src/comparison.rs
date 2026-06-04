use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::csv_reader;
use crate::types::{ColumnMapping, ComparisonConfig, ComparisonResult, SummaryRow};

// ── Internal types ────────────────────────────────────────────────────────────

/// 突合対象の1列分の情報（設定＋CSVインデックス）
struct ColInfo<'a> {
    mapping: &'a ColumnMapping,
    mjs_idx: usize,
    other_idx: usize,
}

/// 明細CSVの1行分のデータ
struct DetailRow {
    key: String,
    mjs_val: String,
    other_val: String,
    diff: String,
}

/// 突合処理の集計結果
struct ComparisonStats {
    mismatch_counts: Vec<usize>,
    detail_entries: Vec<Vec<DetailRow>>,
    mjs_only_count: usize,
    other_only_count: usize,
}

// ── Pure helper functions ─────────────────────────────────────────────────────

fn find_col_index(headers: &[String], name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

/// 2つの値が許容誤差の範囲内で一致するか判定する。
/// tolerance=0 のときは文字列の完全一致、1以上のときは数値変換して絶対差で比較する。
fn values_match(mjs_val: &str, other_val: &str, tolerance: i32) -> bool {
    if tolerance == 0 {
        mjs_val == other_val
    } else {
        match (mjs_val.parse::<f64>(), other_val.parse::<f64>()) {
            (Ok(a), Ok(b)) => (a - b).abs() <= tolerance as f64,
            _ => mjs_val == other_val,
        }
    }
}

/// MJS値 − 他社値 を計算して文字列で返す。数値でない場合は空文字を返す。
fn compute_diff(mjs_val: &str, other_val: &str) -> String {
    match (mjs_val.parse::<f64>(), other_val.parse::<f64>()) {
        (Ok(a), Ok(b)) => {
            let diff = a - b;
            if diff.fract() == 0.0 {
                format!("{}", diff as i64)
            } else {
                format!("{}", diff)
            }
        }
        _ => String::new(),
    }
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn csv_row(fields: &[&str]) -> String {
    fields
        .iter()
        .map(|f| csv_escape(f))
        .collect::<Vec<_>>()
        .join(",")
        + "\n"
}

/// Excelで文字化けしないよう UTF-8 BOM 付きでCSVファイルを書き出す。
fn write_bom_csv(path: &Path, content: &str) -> Result<(), String> {
    let mut bytes: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(content.as_bytes());
    fs::write(path, bytes).map_err(|e| format!("ファイル書き込みエラー: {}", e))
}

// ── Output builders ───────────────────────────────────────────────────────────

fn build_summary_csv(
    compare_cols: &[ColInfo<'_>],
    stats: &ComparisonStats,
) -> (String, Vec<SummaryRow>) {
    let mut content = "項目名,結果\n".to_string();
    let mut rows = Vec::new();

    for (ci, col) in compare_cols.iter().enumerate() {
        let result = if stats.mismatch_counts[ci] == 0 {
            "〇".to_string()
        } else {
            stats.mismatch_counts[ci].to_string()
        };
        content.push_str(&csv_row(&[&col.mapping.mjs_header, &result]));
        rows.push(SummaryRow {
            item_name: col.mapping.mjs_header.clone(),
            result,
        });
    }

    // 片側のみ存在するレコードの集計を末尾に追加
    content.push_str(&csv_row(&["MJS側のみ存在", &stats.mjs_only_count.to_string()]));
    content.push_str(&csv_row(&["他社側のみ存在", &stats.other_only_count.to_string()]));

    (content, rows)
}

fn build_detail_csv(
    key_header: &str,
    compare_cols: &[ColInfo<'_>],
    detail_entries: &[Vec<DetailRow>],
) -> String {
    let mut content = String::new();
    let mut first_section = true;

    for (ci, col) in compare_cols.iter().enumerate() {
        if detail_entries[ci].is_empty() {
            continue;
        }
        if !first_section {
            content.push('\n');
        }
        first_section = false;

        content.push_str(&format!("項目名：{}\n", col.mapping.mjs_header));
        content.push_str(&csv_row(&[key_header, "MJS値", "他社値", "差異"]));

        for row in &detail_entries[ci] {
            content.push_str(&csv_row(&[&row.key, &row.mjs_val, &row.other_val, &row.diff]));
        }
    }

    content
}

// ── Comparison logic ──────────────────────────────────────────────────────────

/// MJSレコードをキー列でインデックス化した HashMap を構築する。
fn build_mjs_index<'a>(
    mjs_records: &'a [Vec<String>],
) -> HashMap<&'a str, &'a Vec<String>> {
    mjs_records
        .iter()
        .filter_map(|rec| rec.first().map(|key| (key.as_str(), rec)))
        .collect()
}

/// 他社CSVの全レコードをMJSのインデックスと突合し、不一致・片側のみ存在を集計する。
fn compare_records(
    mjs_index: &HashMap<&str, &Vec<String>>,
    mjs_records: &[Vec<String>],
    other_records: &[Vec<String>],
    other_key_idx: usize,
    compare_cols: &[ColInfo<'_>],
) -> ComparisonStats {
    let col_count = compare_cols.len();
    let mut mismatch_counts = vec![0usize; col_count];
    let mut detail_entries: Vec<Vec<DetailRow>> = (0..col_count).map(|_| Vec::new()).collect();
    let mut mjs_matched: HashSet<&str> = HashSet::new();
    let mut other_only_count = 0usize;

    for other_record in other_records {
        let key = other_record
            .get(other_key_idx)
            .map(|s| s.as_str())
            .unwrap_or("");

        if let Some(mjs_record) = mjs_index.get(key) {
            mjs_matched.insert(key);
            for (ci, col) in compare_cols.iter().enumerate() {
                let mjs_val = mjs_record.get(col.mjs_idx).map(|s| s.as_str()).unwrap_or("");
                let other_val = other_record.get(col.other_idx).map(|s| s.as_str()).unwrap_or("");
                if !values_match(mjs_val, other_val, col.mapping.tolerance) {
                    mismatch_counts[ci] += 1;
                    detail_entries[ci].push(DetailRow {
                        key: key.to_string(),
                        mjs_val: mjs_val.to_string(),
                        other_val: other_val.to_string(),
                        diff: compute_diff(mjs_val, other_val),
                    });
                }
            }
        } else {
            // 他社側のみに存在するレコード：全比較列に他社値のみを記録する
            other_only_count += 1;
            for (ci, col) in compare_cols.iter().enumerate() {
                let other_val = other_record.get(col.other_idx).map(|s| s.as_str()).unwrap_or("");
                detail_entries[ci].push(DetailRow {
                    key: key.to_string(),
                    mjs_val: String::new(),
                    other_val: other_val.to_string(),
                    diff: String::new(),
                });
            }
        }
    }

    // MJS側のみに存在するレコード：マッチしなかったものを全比較列に記録する
    let mut mjs_only_count = 0usize;
    for record in mjs_records {
        let key = record.first().map(|s| s.as_str()).unwrap_or("");
        if !mjs_matched.contains(key) {
            mjs_only_count += 1;
            for (ci, col) in compare_cols.iter().enumerate() {
                let mjs_val = record.get(col.mjs_idx).map(|s| s.as_str()).unwrap_or("");
                detail_entries[ci].push(DetailRow {
                    key: key.to_string(),
                    mjs_val: mjs_val.to_string(),
                    other_val: String::new(),
                    diff: String::new(),
                });
            }
        }
    }

    ComparisonStats {
        mismatch_counts,
        detail_entries,
        mjs_only_count,
        other_only_count,
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run<F>(config: &ComparisonConfig, on_progress: F) -> Result<ComparisonResult, String>
where
    F: Fn(&str, u32, u32),
{
    on_progress("MJSシステムCSVを読み込み中...", 1, 6);
    let (mjs_headers, mjs_records) =
        csv_reader::read_records(&config.mjs_path, &config.mjs_encoding)?;

    on_progress("他社システムCSVを読み込み中...", 2, 6);
    let (other_headers, other_records) =
        csv_reader::read_records(&config.other_path, &config.other_encoding)?;

    on_progress("突合処理中...", 3, 6);

    // 先頭列をキー列として使用する（仕様: §3.2.3）
    let other_key_col = &config.columns[0].other_header;
    let other_key_idx = find_col_index(&other_headers, other_key_col).ok_or_else(|| {
        format!("他社システムCSVにキー列「{}」が見つかりません", other_key_col)
    })?;

    // 突合対象列のCSVインデックスを解決する
    let compare_cols: Vec<ColInfo> = config.columns[1..]
        .iter()
        .filter(|c| c.is_compare)
        .map(|c| {
            let mjs_idx = find_col_index(&mjs_headers, &c.mjs_header)
                .ok_or_else(|| format!("MJSシステムCSVに列「{}」が見つかりません", c.mjs_header))?;
            let other_idx = find_col_index(&other_headers, &c.other_header)
                .ok_or_else(|| format!("他社システムCSVに列「{}」が見つかりません", c.other_header))?;
            Ok(ColInfo { mapping: c, mjs_idx, other_idx })
        })
        .collect::<Result<_, String>>()?;

    let mjs_index = build_mjs_index(&mjs_records);
    let stats = compare_records(&mjs_index, &mjs_records, &other_records, other_key_idx, &compare_cols);

    on_progress("サマリーファイルを生成中...", 4, 6);
    let (summary_csv, summary_rows) = build_summary_csv(&compare_cols, &stats);

    on_progress("明細ファイルを生成中...", 5, 6);
    let detail_csv = build_detail_csv(&mjs_headers[0], &compare_cols, &stats.detail_entries);

    let summary_path = Path::new(&config.output_dir)
        .join(format!("サマリー比較_{}.csv", config.timestamp));
    let detail_path = Path::new(&config.output_dir)
        .join(format!("明細比較_{}.csv", config.timestamp));

    write_bom_csv(&summary_path, &summary_csv)?;

    let detail_file = if !detail_csv.is_empty() {
        write_bom_csv(&detail_path, &detail_csv)?;
        detail_path.to_string_lossy().to_string()
    } else {
        String::new()
    };

    on_progress("完了", 6, 6);

    Ok(ComparisonResult {
        summary: summary_rows,
        summary_file: summary_path.to_string_lossy().to_string(),
        detail_file,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnMapping;

    fn col(mjs: &str, other: &str, tolerance: i32) -> ColumnMapping {
        ColumnMapping {
            mjs_header: mjs.to_string(),
            other_header: other.to_string(),
            is_compare: true,
            tolerance,
        }
    }

    // ── values_match ──────────────────────────────────────────────────────────

    #[test]
    fn values_match_equal_strings() {
        assert!(values_match("abc", "abc", 0));
    }

    #[test]
    fn values_match_different_strings() {
        assert!(!values_match("abc", "xyz", 0));
    }

    #[test]
    fn values_match_empty_strings_are_equal() {
        assert!(values_match("", "", 0));
    }

    #[test]
    fn values_match_within_tolerance() {
        assert!(values_match("10", "12", 2));
    }

    #[test]
    fn values_match_at_tolerance_boundary() {
        assert!(values_match("10", "12", 2));  // diff=2, ちょうど境界
        assert!(!values_match("10", "13", 2)); // diff=3, 境界超え
    }

    #[test]
    fn values_match_negative_diff_within_tolerance() {
        assert!(values_match("8", "10", 2)); // diff=-2
    }

    #[test]
    fn values_match_non_numeric_with_tolerance_falls_back_to_string() {
        assert!(values_match("abc", "abc", 5));
        assert!(!values_match("abc", "xyz", 5));
    }

    #[test]
    fn values_match_one_side_non_numeric_falls_back_to_string() {
        assert!(!values_match("10", "abc", 5));
    }

    #[test]
    fn values_match_different_numeric_representations_at_zero_tolerance() {
        // "100" と "100.0" は tolerance=0 の文字列比較では不一致
        assert!(!values_match("100", "100.0", 0));
    }

    // ── compute_diff ──────────────────────────────────────────────────────────

    #[test]
    fn compute_diff_positive_integer() {
        assert_eq!(compute_diff("10", "7"), "3");
    }

    #[test]
    fn compute_diff_negative_result() {
        assert_eq!(compute_diff("5", "8"), "-3");
    }

    #[test]
    fn compute_diff_zero() {
        assert_eq!(compute_diff("5", "5"), "0");
    }

    #[test]
    fn compute_diff_float_result() {
        assert_eq!(compute_diff("10.5", "10.0"), "0.5");
    }

    #[test]
    fn compute_diff_both_non_numeric_returns_empty() {
        assert_eq!(compute_diff("abc", "xyz"), "");
    }

    #[test]
    fn compute_diff_one_non_numeric_returns_empty() {
        assert_eq!(compute_diff("10", "abc"), "");
    }

    // ── csv_escape ────────────────────────────────────────────────────────────

    #[test]
    fn csv_escape_plain_text_unchanged() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn csv_escape_empty_string_unchanged() {
        assert_eq!(csv_escape(""), "");
    }

    #[test]
    fn csv_escape_with_comma_wraps_in_quotes() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn csv_escape_with_double_quote_doubles_it() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn csv_escape_with_newline_wraps_in_quotes() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    // ── build_mjs_index ───────────────────────────────────────────────────────

    #[test]
    fn build_mjs_index_maps_key_to_record() {
        let records = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["1002".to_string(), "18".to_string()],
        ];
        let index = build_mjs_index(&records);
        assert_eq!(index.len(), 2);
        assert_eq!(index["1001"][1], "20");
        assert_eq!(index["1002"][1], "18");
    }

    #[test]
    fn build_mjs_index_duplicate_key_keeps_last_record() {
        let records = vec![
            vec!["1001".to_string(), "10".to_string()],
            vec!["1001".to_string(), "99".to_string()], // 後のレコードで上書き
        ];
        let index = build_mjs_index(&records);
        assert_eq!(index.len(), 1);
        assert_eq!(index["1001"][1], "99");
    }

    #[test]
    fn build_mjs_index_empty_records_returns_empty_map() {
        let records: Vec<Vec<String>> = vec![];
        assert!(build_mjs_index(&records).is_empty());
    }

    // ── compare_records ───────────────────────────────────────────────────────

    #[test]
    fn compare_records_all_match() {
        let mapping = col("出勤日数", "WORK_DAYS", 0);
        let info = ColInfo { mapping: &mapping, mjs_idx: 1, other_idx: 1 };
        let mjs = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["1002".to_string(), "18".to_string()],
        ];
        let other = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["1002".to_string(), "18".to_string()],
        ];
        let idx = build_mjs_index(&mjs);
        let stats = compare_records(&idx, &mjs, &other, 0, &[info]);

        assert_eq!(stats.mismatch_counts[0], 0);
        assert_eq!(stats.mjs_only_count, 0);
        assert_eq!(stats.other_only_count, 0);
    }

    #[test]
    fn compare_records_detects_mismatch() {
        let mapping = col("出勤日数", "WORK_DAYS", 0);
        let info = ColInfo { mapping: &mapping, mjs_idx: 1, other_idx: 1 };
        let mjs = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["1002".to_string(), "18".to_string()],
        ];
        let other = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["1002".to_string(), "15".to_string()], // 不一致
        ];
        let idx = build_mjs_index(&mjs);
        let stats = compare_records(&idx, &mjs, &other, 0, &[info]);

        assert_eq!(stats.mismatch_counts[0], 1);
        assert_eq!(stats.detail_entries[0][0].key, "1002");
        assert_eq!(stats.detail_entries[0][0].mjs_val, "18");
        assert_eq!(stats.detail_entries[0][0].other_val, "15");
        assert_eq!(stats.detail_entries[0][0].diff, "3");
    }

    #[test]
    fn compare_records_detects_mjs_only_record() {
        let mapping = col("出勤日数", "WORK_DAYS", 0);
        let info = ColInfo { mapping: &mapping, mjs_idx: 1, other_idx: 1 };
        let mjs = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["9999".to_string(), "5".to_string()], // MJS側のみ
        ];
        let other = vec![
            vec!["1001".to_string(), "20".to_string()],
        ];
        let idx = build_mjs_index(&mjs);
        let stats = compare_records(&idx, &mjs, &other, 0, &[info]);

        assert_eq!(stats.mjs_only_count, 1);
        assert_eq!(stats.other_only_count, 0);
    }

    #[test]
    fn compare_records_detects_other_only_record() {
        let mapping = col("出勤日数", "WORK_DAYS", 0);
        let info = ColInfo { mapping: &mapping, mjs_idx: 1, other_idx: 1 };
        let mjs = vec![
            vec!["1001".to_string(), "20".to_string()],
        ];
        let other = vec![
            vec!["1001".to_string(), "20".to_string()],
            vec!["8888".to_string(), "7".to_string()], // 他社側のみ
        ];
        let idx = build_mjs_index(&mjs);
        let stats = compare_records(&idx, &mjs, &other, 0, &[info]);

        assert_eq!(stats.mjs_only_count, 0);
        assert_eq!(stats.other_only_count, 1);
    }

    #[test]
    fn compare_records_with_tolerance_allows_small_diff() {
        let mapping = col("残業時間", "OT_HOURS", 5);
        let info = ColInfo { mapping: &mapping, mjs_idx: 1, other_idx: 1 };
        let mjs = vec![
            vec!["1001".to_string(), "100".to_string()],
            vec!["1002".to_string(), "50".to_string()],
        ];
        let other = vec![
            vec!["1001".to_string(), "103".to_string()], // 差3：許容内
            vec!["1002".to_string(), "44".to_string()],  // 差6：許容超
        ];
        let idx = build_mjs_index(&mjs);
        let stats = compare_records(&idx, &mjs, &other, 0, &[info]);

        assert_eq!(stats.mismatch_counts[0], 1); // 1002 のみ不一致
    }
}
