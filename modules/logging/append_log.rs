use crate::*;

pub fn append(
    root: impl AsRef<Path>,
    level: char,
    source: &str,
    message: &str,
) -> io::Result<bool> {
    let root = root.as_ref();
    if !allows(root, level) {
        return Ok(false);
    }
    append_line(root, level, source, message)?;
    Ok(true)
}

pub(crate) fn append_line(root: &Path, level: char, source: &str, message: &str) -> io::Result<()> {
    let line = format!("{}\n", format_line(level, source, message));
    with_log_lock(root, || {
        let current = shroudforge_loader_config::paths::current_log(root);
        fs::create_dir_all(
            current
                .parent()
                .ok_or_else(|| io::Error::other("log path has no parent"))?,
        )?;
        let mut file = OpenOptions::new().create(true).append(true).open(current)?;
        file.write_all(line.as_bytes())?;
        file.flush()
    })
}
