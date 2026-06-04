use csv::ReaderBuilder;
use encoding_rs::{SHIFT_JIS, UTF_8};
use std::fs;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn read_headers(path: &str, encoding: &str) -> Result<Vec<String>, String> {
    let text = read_as_text(path, encoding)?;
    let mut reader = make_csv_reader(&text);
    collect_csv_headers(&mut reader)
}

pub fn read_records(
    path: &str,
    encoding: &str,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let text = read_as_text(path, encoding)?;
    let mut reader = make_csv_reader(&text);
    let headers = collect_csv_headers(&mut reader)?;
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSVデータ解析エラー: {}", e))?;
        records.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok((headers, records))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn read_as_text(path: &str, encoding: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("ファイル読み込みエラー: {}", e))?;
    decode_bytes(&bytes, encoding)
}

fn make_csv_reader(text: &str) -> csv::Reader<&[u8]> {
    ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes())
}

fn collect_csv_headers(reader: &mut csv::Reader<&[u8]>) -> Result<Vec<String>, String> {
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| format!("CSVヘッダー解析エラー: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect();
    if headers.is_empty() {
        return Err("ヘッダー行が空です".to_string());
    }
    Ok(headers)
}

pub(crate) fn decode_bytes(bytes: &[u8], encoding: &str) -> Result<String, String> {
    let (cow, _, had_errors) = match encoding {
        "sjis" => SHIFT_JIS.decode(bytes),
        "utf-8" | "utf-8-bom" => UTF_8.decode(bytes),
        other => return Err(format!("未対応の文字コード: {}", other)),
    };
    if had_errors {
        return Err(format!(
            "指定された文字コード({})でファイルを読み込めませんでした",
            encoding
        ));
    }
    // UTF-8 BOM（EF BB BF）が含まれる場合は除去する
    let text = cow.into_owned();
    Ok(text.trim_start_matches('\u{FEFF}').to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_csv(content: &[u8]) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        // ナノ秒を使って並列テスト実行時の衝突を避ける
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        path.push(format!("migrassist_test_{}.csv", nanos));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content).unwrap();
        path
    }

    // ── decode_bytes ──────────────────────────────────────────────────────────

    #[test]
    fn decode_utf8_plain_text() {
        let result = decode_bytes("社員コード,氏名".as_bytes(), "utf-8").unwrap();
        assert_eq!(result, "社員コード,氏名");
    }

    #[test]
    fn decode_utf8_strips_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        bytes.extend_from_slice("社員コード".as_bytes());
        let result = decode_bytes(&bytes, "utf-8-bom").unwrap();
        assert_eq!(result, "社員コード");
    }

    #[test]
    fn decode_unknown_encoding_returns_error() {
        let result = decode_bytes(b"test", "euc-jp");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未対応の文字コード"));
    }

    #[test]
    fn decode_invalid_utf8_sequence_returns_error() {
        // 0x80 は UTF-8 の継続バイトだが先頭には使えない
        let result = decode_bytes(&[0x80, 0x80], "utf-8");
        assert!(result.is_err());
    }

    // ── collect_csv_headers ───────────────────────────────────────────────────

    #[test]
    fn collect_csv_headers_returns_header_row() {
        let text = "社員コード,氏名,出勤日数\n1001,山田,20\n";
        let mut reader = make_csv_reader(text);
        let headers = collect_csv_headers(&mut reader).unwrap();
        assert_eq!(headers, vec!["社員コード", "氏名", "出勤日数"]);
    }

    #[test]
    fn collect_csv_headers_empty_text_returns_error() {
        let mut reader = make_csv_reader("");
        let result = collect_csv_headers(&mut reader);
        assert!(result.is_err());
    }

    // ── read_headers (integration) ────────────────────────────────────────────

    #[test]
    fn read_headers_utf8_file() {
        let path = temp_csv("社員コード,出勤日数\n1001,20\n".as_bytes());
        let headers = read_headers(path.to_str().unwrap(), "utf-8").unwrap();
        assert_eq!(headers, vec!["社員コード", "出勤日数"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn read_headers_nonexistent_file_returns_error() {
        let result = read_headers("/nonexistent/path/file.csv", "utf-8");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ファイル読み込みエラー"));
    }

    // ── read_records (integration) ────────────────────────────────────────────

    #[test]
    fn read_records_returns_headers_and_rows() {
        let path = temp_csv("社員コード,出勤日数\n1001,20\n1002,18\n".as_bytes());
        let (headers, records) = read_records(path.to_str().unwrap(), "utf-8").unwrap();
        assert_eq!(headers, vec!["社員コード", "出勤日数"]);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["1001", "20"]);
        assert_eq!(records[1], vec!["1002", "18"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn read_records_utf8_bom_file_strips_bom() {
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice("社員コード,氏名\n1001,山田\n".as_bytes());
        let path = temp_csv(&content);
        let (headers, records) = read_records(path.to_str().unwrap(), "utf-8-bom").unwrap();
        assert_eq!(headers[0], "社員コード"); // BOM が除去されていること
        assert_eq!(records[0][0], "1001");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn read_records_header_only_returns_empty_records() {
        let path = temp_csv("社員コード,出勤日数\n".as_bytes());
        let (headers, records) = read_records(path.to_str().unwrap(), "utf-8").unwrap();
        assert_eq!(headers.len(), 2);
        assert!(records.is_empty());
        std::fs::remove_file(path).ok();
    }
}
