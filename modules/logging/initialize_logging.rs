use crate::*;

pub fn initialize(root: impl AsRef<Path>, archive_existing: bool) -> io::Result<()> {
    STARTED.get_or_init(Instant::now);
    let root = root.as_ref();
    let current = shroudforge_loader_config::paths::current_log(root);
    fs::create_dir_all(
        current
            .parent()
            .ok_or_else(|| io::Error::other("log path has no parent"))?,
    )?;
    with_log_lock(root, || {
        if archive_existing && current.is_file() && current.metadata()?.len() > 0 {
            let archive = shroudforge_loader_config::paths::logs_dir(root);
            fs::create_dir_all(&archive)?;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            let role = current
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("shroudforge-client");
            let mut destination = archive.join(format!("{role}-{timestamp}.log"));
            for suffix in 2.. {
                if !destination.exists() {
                    break;
                }
                destination = archive.join(format!("{role}-{timestamp}-{suffix}.log"));
            }
            fs::rename(&current, destination)?;
        }
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current)?;
        Ok(())
    })?;
    let config_root = root.to_path_buf();
    let cached = Mutex::new((Instant::now(), configured_level(root)));
    let filter = tracing_subscriber::filter::filter_fn(move |metadata| {
        let Ok(mut cached) = cached.lock() else {
            return false;
        };
        if cached.0.elapsed() >= Duration::from_millis(500) {
            *cached = (Instant::now(), configured_level(&config_root));
        }
        *metadata.level() <= cached.1
    });
    let layer = tracing_subscriber::fmt::layer()
        .event_format(LineFormat)
        .with_writer(LogWriter(Arc::new(current)))
        .with_filter(filter);
    let subscriber = tracing_subscriber::registry().with(layer);
    let _ = subscriber.try_init();
    Ok(())
}
