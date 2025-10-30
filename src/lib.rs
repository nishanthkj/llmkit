use serde_json::{Map, Value};
use std::collections::BTreeMap;

/* ================= Public API ================= */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Unknown,
    Json,
    Ndjson,
    Yaml,
    Toml,
    Csv,
    MarkdownTable,
}

impl DataFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Csv => "csv",
            Self::MarkdownTable => "markdown_table",
        }
    }
}

#[derive(Debug, Clone)]
pub enum TargetFormat {
    Json,
    Yaml,
    Toml,
    Csv,
    MarkdownTable,
    Other(String),
}

impl TargetFormat {
    pub fn name(&self) -> String {
        match self {
            TargetFormat::Json => "json".into(),
            TargetFormat::Yaml => "yaml".into(),
            TargetFormat::Toml => "toml".into(),
            TargetFormat::Csv => "csv".into(),
            TargetFormat::MarkdownTable => "markdown_table".into(),
            TargetFormat::Other(s) => s.clone(),
        }
    }
}

/// Main conversion map. Always returns keys:
/// - "Format", "Original", "Beautified", "normal"
/// - Plus one key per requested target format.
/// If `targets` is None => return **all** formats.
pub fn convert_map(
    input: &[u8],
    targets: Option<&[&str]>,
    allow_permissive: bool,
    max_bytes: Option<usize>,
) -> BTreeMap<String, Value> {
    let mut buf = input.to_vec();
    if let Some(n) = max_bytes {
        if buf.len() > n {
            buf.truncate(n);
        }
    }

    let cleaned = strip_markdown_fences_bytes(&buf);
    let original = String::from_utf8_lossy(&cleaned).to_string();

    let mut out = Map::new();

    if original.trim().is_empty() {
        out.insert("Format".into(), Value::String("unknown".into()));
        out.insert("Original".into(), Value::String(String::new()));
        out.insert("Beautified".into(), Value::String(String::new()));
        out.insert("normal".into(), Value::String(String::new()));
        return out.into_iter().collect();
    }

    match parse_to_value(cleaned.as_slice(), allow_permissive) {
        Ok((val, detected)) => {
            out.insert("Format".into(), Value::String(detected.as_str().into()));
            out.insert("Original".into(), Value::String(original.clone()));

            // Pretty & compact JSON versions
            let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
            let normal = serde_json::to_string(&val).unwrap_or_else(|_| val.to_string());
            out.insert("Beautified".into(), Value::String(pretty));
            out.insert("normal".into(), Value::String(normal));

            let targets = match targets {
                Some(list) => list.iter().map(|s| to_target(s)).collect::<Vec<_>>(),
                None => default_targets(),
            };

            let converted = convert_value_to_formats_with_targets(&val, &targets);
            for (k, v) in converted {
                out.insert(k, v);
            }

            out.into_iter().collect()
        }
        Err(_) => {
            out.insert("Format".into(), Value::String(DataFormat::Unknown.as_str().into()));
            out.insert("Original".into(), Value::String(original.clone()));
            out.insert("Beautified".into(), Value::String(original.clone()));
            out.insert("normal".into(), Value::String(original));
            out.into_iter().collect()
        }
    }
}

/* ================= Helpers ================= */

fn strip_markdown_fences_bytes(input: &[u8]) -> Vec<u8> {
    use regex::Regex;
    let s = String::from_utf8_lossy(input);
    let re_block = Regex::new(r"(?is)```(?:[a-zA-Z0-9_+\-]+)?\s*(.*?)\s*```").unwrap();
    if let Some(cap) = re_block.captures(&s) {
        return cap.get(1).unwrap().as_str().as_bytes().to_vec();
    }
    let re_inline = Regex::new(r"`([^`]*)`").unwrap();
    re_inline.replace_all(&s, "$1").as_bytes().to_vec()
}

fn to_target(s: &str) -> TargetFormat {
    match s.trim().to_lowercase().as_str() {
        "json" => TargetFormat::Json,
        "yaml" => TargetFormat::Yaml,
        "toml" => TargetFormat::Toml,
        "csv" => TargetFormat::Csv,
        "markdown_table" | "md" => TargetFormat::MarkdownTable,
        other => TargetFormat::Other(other.to_string()),
    }
}

fn default_targets() -> Vec<TargetFormat> {
    vec![
        TargetFormat::Json,
        TargetFormat::Yaml,
        TargetFormat::Toml,
        TargetFormat::Csv,
        TargetFormat::MarkdownTable,
    ]
}

fn parse_to_value(input: &[u8], _allow_permissive: bool) -> Result<(Value, DataFormat), ()> {
    let s = String::from_utf8_lossy(input).to_string();

    // JSON
    if let Ok(v) = serde_json::from_str::<Value>(&s) {
        return Ok((v, DataFormat::Json));
    }

    // NDJSON
    if s.lines().count() > 1 {
        let mut arr = Vec::new();
        for line in s.lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                arr.push(v);
            }
        }
        if !arr.is_empty() {
            return Ok((Value::Array(arr), DataFormat::Ndjson));
        }
    }

    // YAML
    #[cfg(feature = "serde_yaml")]
    if let Ok(v) = serde_yaml::from_str::<Value>(&s) {
        return Ok((v, DataFormat::Yaml));
    }

    // TOML
    #[cfg(feature = "toml")]
    if let Ok(tv) = toml::from_str::<toml::Value>(&s) {
        if let Ok(jv) = serde_json::to_value(tv) {
            return Ok((jv, DataFormat::Toml));
        }
    }

    // CSV
    #[cfg(feature = "csv")]
    if s.contains(',') && s.contains('\n') {
        if let Ok(v) = csv_to_json(&s) {
            return Ok((v, DataFormat::Csv));
        }
    }

    // Markdown table
    if let Some(v) = markdown_table_to_json(&s) {
        return Ok((v, DataFormat::MarkdownTable));
    }

    Err(())
}

#[cfg(feature = "csv")]
fn csv_to_json(s: &str) -> Result<Value, ()> {
    let mut rdr = csv::Reader::from_reader(s.as_bytes());
    let mut arr = Vec::new();
    for rec in rdr.deserialize::<serde_json::Value>() {
        arr.push(rec.map_err(|_| ())?);
    }
    Ok(Value::Array(arr))
}

fn markdown_table_to_json(s: &str) -> Option<Value> {
    let lines: Vec<&str> = s.lines().collect();
    let mut start: Option<usize> = None;
    for i in 0..lines.len().saturating_sub(1) {
        if lines[i].contains('|') && lines[i + 1].contains('-') {
            start = Some(i);
            break;
        }
    }
    let idx = start?;
    let headers: Vec<String> = lines[idx]
        .split('|')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
    if headers.is_empty() {
        return None;
    }
    let mut rows = Vec::new();
    for &line in &lines[idx + 2..] {
        if !line.contains('|') {
            break;
        }
        let cells: Vec<String> = line
            .split('|')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect();
        if cells.len() != headers.len() {
            continue;
        }
        let mut obj = serde_json::Map::new();
        for (k, v) in headers.iter().zip(cells.iter()) {
            obj.insert(k.clone(), Value::String(v.clone()));
        }
        rows.push(Value::Object(obj));
    }
    if rows.is_empty() {
        None
    } else {
        Some(Value::Array(rows))
    }
}

pub fn convert_value_to_formats_with_targets(
    v: &Value,
    targets: &[TargetFormat],
) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::<String, Value>::new();
    for tgt in targets {
        let key = tgt.name();
        let val = match tgt {
            TargetFormat::Json => serde_json::to_string_pretty(v).ok().map(Value::String),
            TargetFormat::Yaml => {
                #[cfg(feature = "serde_yaml")]
                { serde_yaml::to_string(v).ok().map(Value::String) }
                #[cfg(not(feature = "serde_yaml"))]
                { None }
            }
            TargetFormat::Toml => {
                #[cfg(feature = "toml")]
                { toml::to_string(v).ok().map(Value::String) }
                #[cfg(not(feature = "toml"))]
                { None }
            }
            TargetFormat::Csv => {
                #[cfg(feature = "csv")]
                { json_to_csv_string(v).ok().map(Value::String) }
                #[cfg(not(feature = "csv"))]
                { None }
            }
            TargetFormat::MarkdownTable => None,
            TargetFormat::Other(_) => None,
        };
        out.insert(key, val.unwrap_or(Value::Null));
    }
    out
}

#[cfg(feature = "csv")]
fn json_to_csv_string(v: &Value) -> Result<String, String> {
    let arr = v.as_array().ok_or_else(|| "CSV requires array of objects".to_string())?;
    let mut headers = BTreeMap::<String, ()>::new();
    for item in arr {
        let obj = item.as_object().ok_or_else(|| "CSV requires array of objects".to_string())?;
        for k in obj.keys() {
            headers.insert(k.clone(), ());
        }
    }
    let headers_vec: Vec<String> = headers.keys().cloned().collect();
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&headers_vec).map_err(|e| e.to_string())?;
    for item in arr {
        let obj = item.as_object().unwrap();
        let row: Vec<String> = headers_vec.iter()
            .map(|h| obj.get(h).map(|v| v.to_string()).unwrap_or_default())
            .collect();
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }
    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/* ============== Python bindings (PyO3) ============== */

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn llmkit_py(_py: Python, m: &PyModule) -> PyResult<()> {
    /// convert_map(input: bytes, targets: list[str] | None, allow_permissive: bool=False, max_input_bytes: int | None=None) -> dict
    #[pyfn(m, "convert_map")]
    fn convert_map_py(
        _py: Python,
        input: &[u8],
        targets: Option<Vec<String>>,
        allow_permissive: bool,
        max_input_bytes: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let t_slices: Option<Vec<&str>> = targets.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
        let map = crate::convert_map(input, t_slices.as_deref(), allow_permissive, max_input_bytes);
        Python::with_gil(|py| Ok(serde_json::to_value(&map).unwrap().into_py(py)))
    }
    Ok(())
}

/* ============== WASM bindings (wasm-bindgen) ============== */

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use serde_wasm_bindgen::to_value as to_js; // <- use serde-wasm-bindgen
    use super::convert_map;

    #[wasm_bindgen]
    pub fn convert_map_js(input: &str, targets: Option<String>, allow_permissive: bool) -> JsValue {
        let targets_vec: Option<Vec<&str>> =
            targets.as_ref().map(|s| s.split(',').map(|x| x.trim()).collect());
        let map = convert_map(input.as_bytes(), targets_vec.as_deref(), allow_permissive, None);

        // Convert serde_json::Value/BTreeMap -> JsValue safely
        to_js(&map).expect("serialize to JsValue failed")
    }
}

