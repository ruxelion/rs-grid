use std::io::Write;
use std::{env, fs, path::PathBuf};

/// Base64 alphabet (standard).
const B64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Minimal base64 encoder — no external dependency.
fn base64_encode(data: &[u8]) -> String {
    let mut out =
        String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[(n >> 18 & 0x3F) as usize] as char);
        out.push(B64[(n >> 12 & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[(n >> 6 & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn main() {
    let flags_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("flags");

    println!("cargo::rerun-if-changed=flags");

    let mut entries: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(&flags_dir).expect("read flags/") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().map(|e| e == "svg") != Some(true) {
            continue;
        }
        let code = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_uppercase();
        let svg_bytes = fs::read(&path).expect("read svg");
        let b64 = base64_encode(&svg_bytes);
        let data_uri =
            format!("data:image/svg+xml;base64,{b64}");
        entries.push((code, data_uri));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("flags_data.rs");
    let mut f =
        fs::File::create(&dest).expect("create output");

    writeln!(
        f,
        "/// {} embedded SVG flags as base64 data URIs.",
        entries.len()
    )
    .unwrap();
    writeln!(
        f,
        "/// Sorted by ISO 3166-1 alpha-2 code for binary \
         search."
    )
    .unwrap();
    writeln!(
        f,
        "static FLAGS: &[(&str, &str)] = &["
    )
    .unwrap();
    for (code, uri) in &entries {
        writeln!(f, "    (\"{code}\", \"{uri}\"),").unwrap();
    }
    writeln!(f, "];").unwrap();
}
