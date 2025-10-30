use std::io::{self, Read};
use std::{env, fs, process};
use serde_json::Value;
use llmkit::convert_map;

fn main() {
    let mut file_path: Option<String> = None;
    let mut targets_arg: Option<String> = None;
    let mut single_format: Option<String> = None;
    let mut allow_permissive = false;
    let mut max_bytes: Option<usize> = None;

    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--file" => file_path = args.next(),
            "--targets" => targets_arg = args.next(),
            "--format" => single_format = args.next(),
            "--permissive" => allow_permissive = true,
            "--max-bytes" => max_bytes = args.next().and_then(|n| n.parse::<usize>().ok()),
            "--help" | "-h" => usage(),
            _ => usage(),
        }
    }

    let mut input = match file_path {
        Some(p) => fs::read(&p).expect("failed to read file"),
        None => {
            let mut b = Vec::new();
            io::stdin().read_to_end(&mut b).expect("stdin read failed");
            b
        }
    };

    if let Some(n) = max_bytes {
        if input.len() > n { input.truncate(n); }
    }

    // Choose targets
    let targets: Option<Vec<&str>> = if let Some(fmt) = single_format {
        Some(vec![fmt.as_str()])
    } else {
        targets_arg
            .as_ref()
            .map(|s| s.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()).collect())
    };

    let map = convert_map(&input, targets.as_deref(), allow_permissive, max_bytes);
    let json_obj: Value = Value::Object(map.into_iter().collect());
    println!("{}", serde_json::to_string_pretty(&json_obj).unwrap());
}

fn usage() -> ! {
    eprintln!(
        "usage: llmkit [--file <path>] [--targets json,yaml,...] [--format yaml] [--permissive] [--max-bytes N]"
    );
    process::exit(2);
}
