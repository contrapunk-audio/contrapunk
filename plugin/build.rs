use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CONTRAPUNK_PLUGIN_UI_DIR");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ui_dir = env::var_os("CONTRAPUNK_PLUGIN_UI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../ui/build"));

    println!("cargo:rerun-if-changed={}", ui_dir.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("ui_assets.rs");

    let mut entries = Vec::new();
    if ui_dir.exists() {
        collect_files(&ui_dir, &ui_dir, &mut entries);
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut generated =
        String::from("pub(super) fn get_asset(path: &str) -> Option<&'static [u8]> {\n");
    generated.push_str("    match path.trim_start_matches('/') {\n");
    for (route, file) in entries {
        println!("cargo:rerun-if-changed={}", file.display());
        generated.push_str(&format!(
            "        {:?} => Some(include_bytes!({:?}) as &'static [u8]),\n",
            route,
            file.display().to_string()
        ));
    }
    generated.push_str("        _ => None,\n");
    generated.push_str("    }\n}\n");

    fs::write(out_file, generated).unwrap();
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, path));
        }
    }
}
