#[test]
fn empty_input_returns_unknown() {
    let map = llmkit::convert_map(b"", None, false, None);
    assert_eq!(map.get("Format").unwrap(), "unknown");
    assert_eq!(map.get("Original").unwrap(), "");
    assert_eq!(map.get("Beautified").unwrap(), "");
    assert_eq!(map.get("normal").unwrap(), "");
    assert_eq!(map.len(), 4);
}

#[test]
fn json_input_detects_and_beautifies_and_normal() {
    let map = llmkit::convert_map(br#"{"a":1,"b":"x"}"#, None, false, None);
    assert_eq!(map.get("Format").unwrap(), "json");
    assert!(map.get("Beautified").unwrap().as_str().unwrap().contains("\n"));
    assert_eq!(map.get("normal").unwrap(), "{\"a\":1,\"b\":\"x\"}");
}

#[test]
fn markdown_table_detects() {
    let md = b"|a|b|\n|--|--|\n|1|x|\n";
    let map = llmkit::convert_map(md, None, false, None);
    assert_eq!(map.get("Format").unwrap(), "markdown_table");
}

#[test]
fn only_requested_targets_are_included() {
    let map = llmkit::convert_map(br#"{"a":1}"#, Some(&["json"]), false, None);
    assert!(map.contains_key("json"));
    assert!(!map.contains_key("yaml"));
}
