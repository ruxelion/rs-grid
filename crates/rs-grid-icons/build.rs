use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Base64 alphabet (standard).
const B64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Minimal base64 encoder — no external dependency.
fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
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

/// Scan a directory for `.svg` files, base64-encode each,
/// return sorted `(KEY, data_uri)` pairs.
/// KEY = file stem in UPPERCASE.
fn scan_svg_dir(dir: &Path) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = Vec::new();
    let reader = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return entries,
    };
    for entry in reader {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().map(|e| e == "svg") != Some(true) {
            continue;
        }
        let key = path.file_stem().unwrap().to_str().unwrap().to_uppercase();
        let svg_bytes = fs::read(&path).expect("read svg");
        let b64 = base64_encode(&svg_bytes);
        let data_uri = format!("data:image/svg+xml;base64,{b64}");
        entries.push((key, data_uri));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

/// Write a static array to the output file.
fn write_array(f: &mut fs::File, name: &str, entries: &[(String, String)]) {
    writeln!(f, "static {name}: &[(&str, &str)] = &[").unwrap();
    for (key, uri) in entries {
        writeln!(f, "    (\"{key}\", \"{uri}\"),").unwrap();
    }
    writeln!(f, "];").unwrap();
}

/// Parse a flat `CODE = "Name"` TOML file into sorted
/// (code, name) pairs.
fn parse_country_names_file(path: &Path) -> Vec<(String, String)> {
    let src = fs::read_to_string(path).expect("read country names toml");
    let mut entries = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        let val = rest.trim();
        let val = val
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or(val);
        entries.push((key.trim().to_string(), val.to_string()));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

/// Escape a string for embedding as a Rust string literal —
/// needed here (unlike the base64 flag/gender data above)
/// because these are free-text translations from an external
/// source.
fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo::rerun-if-changed=flags");
    println!("cargo::rerun-if-changed=genders");
    println!("cargo::rerun-if-changed=country_names");

    let flags = scan_svg_dir(&manifest_dir.join("flags"));
    let genders = scan_svg_dir(&manifest_dir.join("genders"));

    let mut translations: Vec<(String, String, String)> = Vec::new();
    if let Ok(reader) = fs::read_dir(manifest_dir.join("country_names")) {
        for entry in reader {
            let path = entry.expect("dir entry").path();
            if path.extension().map(|e| e == "toml") != Some(true) {
                continue;
            }
            let lang =
                path.file_stem().unwrap().to_str().unwrap().to_lowercase();
            for (code, name) in parse_country_names_file(&path) {
                translations.push((lang.clone(), code, name));
            }
        }
    }
    translations.sort();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("icons_data.rs");
    let mut f = fs::File::create(&dest).expect("create output");

    write_array(&mut f, "FLAGS", &flags);
    writeln!(f).unwrap();
    write_array(&mut f, "GENDERS", &genders);
    writeln!(f).unwrap();

    writeln!(f, "static COUNTRY_TRANSLATIONS: &[(&str, &str, &str)] = &[")
        .unwrap();
    for (lang, code, name) in &translations {
        writeln!(
            f,
            "    (\"{lang}\", \"{code}\", \"{}\"),",
            escape_rust_str(name)
        )
        .unwrap();
    }
    writeln!(f, "];").unwrap();
}
