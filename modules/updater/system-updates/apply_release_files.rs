use crate::validate_update_paths::installed_path;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const VERSION_MANIFEST: &str = "shroudforge/version.json";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const COPY_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const COPY_BUFFER_SIZE: usize = 1024 * 1024;

#[cfg(test)]
pub(crate) fn apply(source: &Path, target: &Path, paths: &[String]) -> Result<(), String> {
    apply_with_progress(source, target, paths, |_, _, _, _, _| Ok(()))
}

#[cfg(test)]
pub(crate) fn apply_with_progress(
    source: &Path,
    target: &Path,
    paths: &[String],
    progress: impl FnMut(usize, usize, &str, u64, u64) -> Result<(), String>,
) -> Result<(), String> {
    apply_with_progress_and_cancel(source, target, paths, None, progress)
}

pub(crate) fn apply_with_progress_and_cancel(
    source: &Path,
    target: &Path,
    paths: &[String],
    cancel_root: Option<&Path>,
    mut progress: impl FnMut(usize, usize, &str, u64, u64) -> Result<(), String>,
) -> Result<(), String> {
    let mut version_manifest = false;
    let total = paths.len();
    let mut completed = 0;
    for relative in paths {
        if relative == VERSION_MANIFEST {
            version_manifest = true;
            continue;
        }
        let incoming = source.join(relative);
        if !incoming.exists() {
            return Err(format!("release is missing managed path: {relative}"));
        }
        let current = installed_path(target, relative);
        copy_file_with_progress_and_cancel(
            &incoming,
            &current,
            cancel_root,
            |bytes, file_total| progress(completed, total, relative, bytes, file_total),
        )?;
        completed += 1;
        progress(completed, total, "", 0, 0)?;
    }

    // Publish the manifest only after every other managed file is in place.
    // Readers can then observe the old complete manifest or the new complete
    // manifest, never the truncated file produced by copying over it directly.
    if version_manifest {
        let incoming = source.join(VERSION_MANIFEST);
        if !incoming.is_file() {
            return Err(format!(
                "release is missing managed path: {VERSION_MANIFEST}"
            ));
        }
        let current = installed_path(target, VERSION_MANIFEST);
        copy_file_atomically(&incoming, &current)?;
        completed += 1;
        progress(completed, total, "", 0, 0)?;
    }
    Ok(())
}

pub(crate) fn copy_file_with_progress(
    source: &Path,
    target: &Path,
    progress: impl FnMut(u64, u64) -> Result<(), String>,
) -> Result<u64, String> {
    copy_file_with_progress_and_cancel(source, target, None, progress)
}

pub(crate) fn copy_file_with_progress_and_cancel(
    source: &Path,
    target: &Path,
    cancel_root: Option<&Path>,
    mut progress: impl FnMut(u64, u64) -> Result<(), String>,
) -> Result<u64, String> {
    let watchdog = CopyWatchdog::start(cancel_root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "could not create parent directory {} for update file {}: {error}",
                parent.display(),
                target.display()
            )
        })?;
    }
    if let Err(error) = watchdog.check() {
        return Err(if watchdog.is_cancelled() {
            "UPDATE_CANCELLED".into()
        } else {
            copy_error(source, target, error)
        });
    }
    for attempt in 0..10 {
        match copy_file_attempt(source, target, &watchdog, &mut progress) {
            Err(error) if retryable_file_lock(&error) && attempt < 9 => {
                std::thread::sleep(Duration::from_millis(200));
                if let Err(error) = watchdog.check() {
                    return Err(if watchdog.is_cancelled() {
                        "UPDATE_CANCELLED".into()
                    } else {
                        copy_error(source, target, error)
                    });
                }
                watchdog.record_progress();
            }
            Err(error) if error.to_string() == "UPDATE_CANCELLED" || watchdog.is_cancelled() => {
                return Err("UPDATE_CANCELLED".into());
            }
            Ok(copied) => return Ok(copied),
            Err(error) => {
                if let Err(watchdog_error) = watchdog.check() {
                    return Err(if watchdog.is_cancelled() {
                        "UPDATE_CANCELLED".into()
                    } else {
                        copy_error(source, target, watchdog_error)
                    });
                }
                return Err(copy_error(source, target, error));
            }
        }
    }
    unreachable!()
}

fn copy_file_attempt(
    source: &Path,
    target: &Path,
    watchdog: &CopyWatchdog,
    progress: &mut impl FnMut(u64, u64) -> Result<(), String>,
) -> io::Result<u64> {
    watchdog.check()?;
    let metadata = fs::metadata(source)?;
    let total = metadata.len();
    let mut input = File::open(source)?;
    watchdog.check()?;
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target)?;
    watchdog.check()?;
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
    let mut copied = 0u64;
    loop {
        watchdog.check()?;
        let count = input.read(&mut buffer)?;
        watchdog.check()?;
        if count == 0 {
            break;
        }
        output.write_all(&buffer[..count])?;
        watchdog.check()?;
        copied = copied.saturating_add(count as u64);
        watchdog.record_progress();
        progress(copied, total).map_err(io::Error::other)?;
    }
    output.flush()?;
    fs::set_permissions(target, metadata.permissions())?;
    watchdog.check()?;
    Ok(copied)
}

fn copy_error(source: &Path, target: &Path, error: io::Error) -> String {
    if error.kind() == io::ErrorKind::TimedOut {
        format!(
            "update file copy made no progress for {} seconds ({} to {})",
            COPY_IDLE_TIMEOUT.as_secs(),
            source.display(),
            target.display()
        )
    } else {
        format!(
            "could not copy update file {} to {}: {error}",
            source.display(),
            target.display()
        )
    }
}

struct CopyWatchdog {
    started: Instant,
    last_progress_ms: Arc<AtomicU64>,
    timed_out: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    stop: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl CopyWatchdog {
    fn start(cancel_root: Option<&Path>) -> Self {
        Self::start_with_timeout(cancel_root, COPY_IDLE_TIMEOUT)
    }

    fn start_with_timeout(cancel_root: Option<&Path>, idle_timeout: Duration) -> Self {
        let started = Instant::now();
        let last_progress_ms = Arc::new(AtomicU64::new(0));
        let timed_out = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        let (stop, receiver) = mpsc::channel();
        let last_progress = last_progress_ms.clone();
        let timed_out_signal = timed_out.clone();
        let cancelled_signal = cancelled.clone();
        let cancel_root = cancel_root.map(Path::to_path_buf);
        if cancel_root.as_deref().is_some_and(|root| {
            crate::check_update_cancelled::check_update_cancelled(root).is_err()
        }) {
            cancelled.store(true, Ordering::Release);
        }
        #[cfg(windows)]
        let thread_id = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
        let thread = std::thread::spawn(move || {
            loop {
                match receiver.recv_timeout(Duration::from_millis(250).min(idle_timeout)) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                let cancel_requested = cancel_root.as_deref().is_some_and(|root| {
                    crate::check_update_cancelled::check_update_cancelled(root).is_err()
                });
                if cancel_requested {
                    cancelled_signal.store(true, Ordering::Release);
                    #[cfg(windows)]
                    cancel_synchronous_io(thread_id);
                    break;
                }
                let now_ms = started.elapsed().as_millis() as u64;
                let previous_ms = last_progress.load(Ordering::Acquire);
                if now_ms.saturating_sub(previous_ms) < idle_timeout.as_millis() as u64 {
                    continue;
                }
                timed_out_signal.store(true, Ordering::Release);
                #[cfg(windows)]
                cancel_synchronous_io(thread_id);
                break;
            }
        });
        Self {
            started,
            last_progress_ms,
            timed_out,
            cancelled,
            stop,
            thread: Some(thread),
        }
    }

    fn record_progress(&self) {
        self.last_progress_ms
            .store(self.started.elapsed().as_millis() as u64, Ordering::Release);
    }

    fn check(&self) -> io::Result<()> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "UPDATE_CANCELLED",
            ));
        }
        if self.timed_out.load(Ordering::Acquire) {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "update file copy exceeded its idle timeout",
            ))
        } else {
            Ok(())
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(windows)]
fn cancel_synchronous_io(thread_id: u32) {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::{
            IO::CancelSynchronousIo,
            Threading::{OpenThread, THREAD_TERMINATE},
        },
    };
    unsafe {
        let handle = OpenThread(THREAD_TERMINATE, 0, thread_id);
        if !handle.is_null() {
            let _ = CancelSynchronousIo(handle);
            CloseHandle(handle);
        }
    }
}

pub(crate) fn copy_reader_with_cancellation<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    cancel_root: &Path,
    description: &str,
) -> Result<u64, String> {
    let watchdog = CopyWatchdog::start(Some(cancel_root));
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    let mut copied = 0_u64;
    loop {
        watchdog.check().map_err(|error| {
            if watchdog.is_cancelled() {
                "UPDATE_CANCELLED".to_owned()
            } else if error.kind() == io::ErrorKind::TimedOut {
                format!(
                    "{description} made no progress for {} seconds",
                    COPY_IDLE_TIMEOUT.as_secs()
                )
            } else {
                format!("{description} was interrupted: {error}")
            }
        })?;
        let count = match reader.read(&mut buffer) {
            Ok(count) => count,
            Err(_) if watchdog.is_cancelled() => return Err("UPDATE_CANCELLED".into()),
            Err(error) => {
                if let Err(watchdog_error) = watchdog.check() {
                    if watchdog.is_cancelled() {
                        return Err("UPDATE_CANCELLED".into());
                    }
                    if watchdog_error.kind() == io::ErrorKind::TimedOut {
                        return Err(format!(
                            "{description} made no progress for {} seconds",
                            COPY_IDLE_TIMEOUT.as_secs()
                        ));
                    }
                }
                return Err(format!("{description} failed while reading: {error}"));
            }
        };
        watchdog.check().map_err(|_| {
            if watchdog.is_cancelled() {
                "UPDATE_CANCELLED".to_owned()
            } else {
                format!(
                    "{description} made no progress for {} seconds",
                    COPY_IDLE_TIMEOUT.as_secs()
                )
            }
        })?;
        if count == 0 {
            return Ok(copied);
        }
        if let Err(error) = writer.write_all(&buffer[..count]) {
            if watchdog.is_cancelled() {
                return Err("UPDATE_CANCELLED".into());
            }
            return Err(format!("{description} failed while writing: {error}"));
        }
        copied = copied.saturating_add(count as u64);
        watchdog.record_progress();
    }
}

impl Drop for CopyWatchdog {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn copy_file_atomically(source: &Path, target: &Path) -> Result<(), String> {
    let parent = target.parent().ok_or_else(|| {
        format!(
            "version manifest has no parent directory: {}",
            target.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "could not create parent directory {} for version manifest {}: {error}",
            parent.display(),
            target.display()
        )
    })?;
    let source_metadata = fs::metadata(source).map_err(|error| {
        format!(
            "could not inspect staged version manifest {}: {error}",
            source.display()
        )
    })?;
    let mut source_file = File::open(source).map_err(|error| {
        format!(
            "could not read staged version manifest {}: {error}",
            source.display()
        )
    })?;
    let file_name = target
        .file_name()
        .ok_or_else(|| format!("version manifest has no file name: {}", target.display()))?;

    let (temporary, mut destination_file) = (0..32)
        .find_map(|_| {
            let mut name = OsString::from(".");
            name.push(file_name);
            name.push(format!(
                ".{}.{}.tmp",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let temporary = parent.join(name);
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => Some(Ok((temporary, file))),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => None,
                Err(error) => Some(Err(error)),
            }
        })
        .transpose()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "could not allocate a temporary version manifest".to_owned())?;

    let result = (|| -> io::Result<()> {
        io::copy(&mut source_file, &mut destination_file)?;
        destination_file.sync_all()?;
        drop(destination_file);
        fs::set_permissions(&temporary, source_metadata.permissions())?;
        replace_file_atomically_with_retry(&temporary, target)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "could not publish version manifest {} atomically: {error}",
            target.display()
        ));
    }
    Ok(())
}

fn replace_file_atomically_with_retry(source: &Path, target: &Path) -> io::Result<()> {
    const ATTEMPTS: usize = 10;
    for attempt in 0..ATTEMPTS {
        match replace_file_atomically(source, target) {
            Err(error) if retryable_file_lock(&error) && attempt + 1 < ATTEMPTS => {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            result => return result,
        }
    }
    unreachable!()
}

#[cfg(windows)]
fn replace_file_atomically(source: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file_atomically(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
}

fn retryable_file_lock(error: &std::io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock || matches!(error.raw_os_error(), Some(5 | 32 | 33))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "shroudforge-{label}-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn manifest_is_not_published_when_another_managed_file_fails() {
        let root = temporary_root("manifest-order");
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("shroudforge")).unwrap();
        fs::create_dir_all(target.join("shroudforge")).unwrap();
        fs::write(source.join(VERSION_MANIFEST), b"new manifest").unwrap();
        fs::write(target.join(VERSION_MANIFEST), b"old manifest").unwrap();

        let result = apply(
            &source,
            &target,
            &[
                VERSION_MANIFEST.to_owned(),
                "missing-managed-file.dll".to_owned(),
            ],
        );

        assert!(result.is_err());
        assert_eq!(
            fs::read(target.join(VERSION_MANIFEST)).unwrap(),
            b"old manifest"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_replacement_publishes_complete_new_contents() {
        let root = temporary_root("manifest-atomic");
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir_all(source.join("shroudforge")).unwrap();
        fs::create_dir_all(target.join("shroudforge")).unwrap();
        fs::write(source.join(VERSION_MANIFEST), b"new complete manifest").unwrap();
        fs::write(target.join(VERSION_MANIFEST), b"old complete manifest").unwrap();

        apply(&source, &target, &[VERSION_MANIFEST.to_owned()]).unwrap();

        assert_eq!(
            fs::read(target.join(VERSION_MANIFEST)).unwrap(),
            b"new complete manifest"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancellation_from_progress_callback_keeps_the_cancelled_result() {
        let root = temporary_root("copy-cancel");
        let source = root.join("source.bin");
        let target = root.join("target.bin");
        fs::create_dir_all(&root).unwrap();
        fs::write(&source, vec![7_u8; 1024]).unwrap();

        let result =
            copy_file_with_progress(&source, &target, |_, _| Err("UPDATE_CANCELLED".into()));

        assert_eq!(result.unwrap_err(), "UPDATE_CANCELLED");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn copy_watchdog_reports_a_stalled_operation() {
        let watchdog = CopyWatchdog::start_with_timeout(None, Duration::from_millis(20));
        let deadline = Instant::now() + Duration::from_secs(1);
        while watchdog.check().is_ok() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(
            watchdog.check().unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }
}
