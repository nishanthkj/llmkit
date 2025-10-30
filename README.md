# LLMKit

**LLMKit** — cross-language toolkit to **clean, validate, and convert** LLM outputs into structured formats.

Use it as a:

- **Rust library**
- **CLI**
- **Python package (PyO3)**
- **NPM/WASM module**

---

## ✨ Features

| Feature          | What you get                                                              |
| ---------------- | ------------------------------------------------------------------------- |
| Rust core        | Fast, type-safe detection + conversion                                    |
| CLI              | Convert stdin/files to JSON/YAML/TOML/CSV/etc.                            |
| Python (PyO3)    | Drop-in for Python data workflows                                         |
| JS/WASM          | Use in Node, Bun, or browsers                                             |
| Auto-detect      | JSON, NDJSON, YAML, TOML, CSV, Markdown tables                            |
| Multi-output     | **All formats by default**; filter via args                               |
| Compact + pretty | Always returns `Beautified` (pretty JSON) and `normal` (single-line JSON) |

---

## 🧱 Architecture

```
 Core (detect, parse, convert)
            │
   ┌────────┼─────────┐
   │        │         │
  CLI     PyO3      WASM
 cargo     pip       npm/web
```

---

## ⚙️ Installation

### Rust (library)

```toml
# Cargo.toml
[dependencies]
llmkit = { git = "https://github.com/yourname/llmkit", features = ["csv", "serde_yaml", "toml"] }
```

```rust
use llmkit::convert_map;

fn main() {
    let out = convert_map(br#"{"user":"nishanth","age":25}"#, None, false, None);
    // out is BTreeMap<String, serde_json::Value>
    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out.into_iter().collect())).unwrap());
}
```

### CLI

```bash
cargo build --features "csv serde_yaml toml"

# All formats (default)
echo '{"a":1,"b":"x"}' | cargo run --features "csv serde_yaml toml"

# Only JSON + YAML
echo '{"a":1,"b":"x"}' | cargo run --features "csv serde_yaml toml" -- --targets json,yaml

# Exactly one format (shortcut)
echo '{"a":1,"b":"x"}' | cargo run -- --format yaml
```

**Flags**

| Flag                      | Meaning                                        |
| ------------------------- | ---------------------------------------------- |
| `--file <path>`           | Read from file (otherwise stdin)               |
| `--targets json,yaml,...` | Return only these formats                      |
| `--format <fmt>`          | Return only one format (overrides `--targets`) |
| `--permissive`            | (reserved) looser parsing                      |
| `--max-bytes N`           | Truncate input to N bytes                      |
| `-h`, `--help`            | Usage                                          |

**Example output**

```json
{
  "Format": "json",
  "Original": "{\"a\":1,\"b\":\"x\"}",
  "Beautified": "{\n  \"a\": 1,\n  \"b\": \"x\"\n}",
  "normal": "{\"a\":1,\"b\":\"x\"}",
  "json": "{\n  \"a\": 1,\n  \"b\": \"x\"\n}",
  "yaml": "a: 1\nb: x\n",
  "toml": "a = 1\nb = \"x\"\n"
}
```

### Python (pip via maturin)

```bash
pip install maturin
maturin build --release -F python
pip install target/wheels/llmkit-*.whl
```

```python
import llmkit_py as m

# All formats
print(m.convert_map(b'{"language":"Rust","creator":"Graydon"}', None, False, None))

# Specific formats (list of strings)
print(m.convert_map(b'{"a":1}', ["json","yaml"], False, None))
```

_Always includes_ `Format`, `Original`, `Beautified`, `normal`, plus per-format keys.

### NPM / WASM

```bash
cargo install wasm-pack
npm run build        # bundler target
# or: npm run build:node / npm run build:web
```

```js
import init, { convert_map_js } from "./pkg/llmkit.js";

await init();

// All formats (pass null) or a CSV list of targets
const all = convert_map_js('{"a":1}', null, false);
const only = convert_map_js('{"a":1}', "json,yaml", false);

console.log(all.normal); // => {"a":1}
```

---

## 🧩 Supported Formats

| Format         | Detects                    | Converts to                                |
| -------------- | -------------------------- | ------------------------------------------ |
| JSON           | Leading `{` / `[`          | JSON, YAML, TOML, CSV\*, MarkdownTable\*\* |
| NDJSON         | Multiple JSON lines        | JSON array                                 |
| YAML\*         | `key: value`/indentation   | JSON/TOML                                  |
| TOML\*         | `[section]`, `key = value` | JSON/YAML                                  |
| CSV\*          | Commas + stable columns    | JSON array                                 |
| Markdown Table | header + `---` separator   | JSON array                                 |

- enable related Cargo features (e.g., `serde_yaml`, `toml`, `csv`)
  \*\* generation of Markdown is not implemented (parsing is)

**Heuristics (quick)**

- Starts with `{`/`[` → JSON; many JSON lines → NDJSON
- `key: value` + indentation → YAML
- `[header]` lines → TOML
- Commas + consistent columns → CSV
- Header row + `|---|` separator → Markdown table

---

## 🧪 Tests

```bash
cargo test --features "csv serde_yaml toml"
```

Covers: JSON/NDJSON/Markdown detection, CLI flags, and presence of `normal`.

---

## 🧰 Dev Commands

| Task         | Command                                            |
| ------------ | -------------------------------------------------- |
| Build CLI    | `cargo build --features "csv serde_yaml toml"`     |
| Run CLI      | `cargo run --features "csv serde_yaml toml"`       |
| Tests        | `cargo test --features "csv serde_yaml toml"`      |
| Python wheel | `maturin build --release -F python`                |
| WASM         | `wasm-pack build --target bundler --features wasm` |

---

## 🔚 End-to-end example (fenced JSON)

Input:

````markdown
```json
{ "a": 1, "b": "x" }
```
````

Output (truncated):

```json
{
  "Format": "json",
  "Original": "{\"a\":1,\"b\":\"x\"}",
  "Beautified": "{\n  \"a\": 1,\n  \"b\": \"x\"\n}",
  "normal": "{\"a\":1,\"b\":\"x\"}",
  "json": "{\n  \"a\": 1,\n  \"b\": \"x\"\n}",
  "yaml": "a: 1\nb: x\n"
}
```

---

## 🧾 License

MIT OR Apache-2.0

## 👤 Author

Built by **Nishanth** — focused on developer productivity and data sanity.
