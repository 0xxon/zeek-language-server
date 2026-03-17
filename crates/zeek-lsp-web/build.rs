use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

/// Try to discover the Zeek scripts directory via `zeek-config --prefix`.
fn zeek_scripts_from_config() -> Option<String> {
    let output = Command::new("zeek-config").arg("--prefix").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let prefix = String::from_utf8(output.stdout).ok()?.trim().to_string();
    let path = PathBuf::from(&prefix).join("share/zeek");
    if path.exists() {
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn main() {
    let scripts_dir = env::var("ZEEK_SCRIPTS_DIR").ok()
        .or_else(zeek_scripts_from_config)
        .expect("Could not find Zeek scripts directory. Set ZEEK_SCRIPTS_DIR or ensure zeek-config is in PATH.");
    let scripts_path = Path::new(&scripts_dir);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let stdlib_rs = out_dir.join("stdlib.rs");
    let mut out = File::create(&stdlib_rs).expect("cannot create stdlib.rs");

    writeln!(out, "pub static STDLIB_FILES: &[(&str, &str)] = &[").unwrap();

    assert!(scripts_path.exists(), "Zeek scripts directory does not exist: {scripts_dir}");
    collect_zeek_files(scripts_path, scripts_path, &mut out);

    writeln!(out, "];").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=ZEEK_SCRIPTS_DIR");
}

fn collect_zeek_files(base: &Path, dir: &Path, out: &mut File) {
    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() && path.file_name().and_then(|s| s.to_str()) != Some("cmake") {
            collect_zeek_files(base, &path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("zeek") {
            let rel = path.strip_prefix(base).unwrap();
            let virtual_path = format!("/zeek/scripts/{}", rel.display());
            let abs = path.canonicalize().unwrap_or(path.clone());
            writeln!(
                out,
                "    ({:?}, include_str!({:?})),",
                virtual_path,
                abs.display().to_string()
            )
            .unwrap();
        }
    }
}
