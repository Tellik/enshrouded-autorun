use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() && entry.file_name() != "__pycache__" {
            collect(&path, files)?;
        } else if kind.is_file() {
            let extension = path.extension().and_then(|value| value.to_str());
            let requirements = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("requirements") && name.ends_with(".txt"));
            if matches!(extension, Some("json" | "py" | "lua")) || requirements {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let directory =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("diagnostics-jobs");
    let mut files = Vec::new();
    collect(&directory, &mut files)?;
    files.sort();
    let mut output = String::from("const BUNDLED_FILES: &[(&str, &str)] = &[\n");
    for path in files {
        let relative = path
            .strip_prefix(&directory)
            .unwrap()
            .to_str()
            .expect("diagnostic bundle path is UTF-8")
            .replace('\\', "/");
        let source = path.to_str().expect("diagnostic bundle source is UTF-8");
        output.push_str(&format!("    ({relative:?}, include_str!({source:?})),\n"));
    }
    output.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("diagnostic_jobs.rs"),
        output,
    )
}
