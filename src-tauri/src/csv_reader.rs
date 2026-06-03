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

fn decode_bytes(bytes: &[u8], encoding: &str) -> Result<String, String> {
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
