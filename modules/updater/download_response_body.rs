use std::{path::Path, time::Duration};

const CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(200);

pub(crate) fn download_response_body(
    root: &Path,
    request: reqwest::RequestBuilder,
    maximum_bytes: u64,
    idle_timeout: Duration,
    description: &str,
    oversized_error: &str,
    mut on_response: impl FnMut(Option<u64>) -> Result<(), String>,
    mut on_chunk: impl FnMut(&[u8], Option<u64>) -> Result<(), String>,
) -> Result<(u64, Option<u64>), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("could not start the download runtime: {error}"))?;

    runtime.block_on(async {
        let mut cancellation_checks = tokio::time::interval(CANCELLATION_POLL_INTERVAL);
        cancellation_checks.tick().await;
        crate::check_update_cancelled(root)?;

        let mut last_progress = std::time::Instant::now();
        let send = request.send();
        tokio::pin!(send);
        let (mut response, content_length) = loop {
            tokio::select! {
                result = &mut send => {
                    crate::check_update_cancelled(root)?;
                    let response = result.map_err(|error| {
                        format!("{description} request failed: {error}")
                    })?;
                    if !response.status().is_success() {
                        return Err(format!(
                            "{description} returned HTTP {}",
                            response.status()
                        ));
                    }
                    let content_length = response.content_length();
                    if content_length.is_some_and(|length| length > maximum_bytes) {
                        return Err(oversized_error.into());
                    }
                    on_response(content_length)?;
                    last_progress = std::time::Instant::now();
                    break (response, content_length);
                }
                _ = cancellation_checks.tick() => {
                    crate::check_update_cancelled(root)?;
                    if last_progress.elapsed() >= idle_timeout {
                        return Err(format!(
                            "{description} made no progress for {} seconds",
                            idle_timeout.as_secs()
                        ));
                    }
                }
            }
        };

        let mut received = 0u64;
        loop {
            tokio::select! {
                chunk = response.chunk() => {
                    match chunk {
                        Ok(Some(chunk)) => {
                            crate::check_update_cancelled(root)?;
                            received = received.saturating_add(chunk.len() as u64);
                            if received > maximum_bytes {
                                return Err(oversized_error.into());
                            }
                            on_chunk(&chunk, content_length)?;
                            last_progress = std::time::Instant::now();
                        }
                        Ok(None) => break,
                        Err(error) => {
                            crate::check_update_cancelled(root)?;
                            return Err(format!("{description} failed while reading: {error}"));
                        }
                    }
                }
                _ = cancellation_checks.tick() => {
                    crate::check_update_cancelled(root)?;
                    if last_progress.elapsed() >= idle_timeout {
                        return Err(format!(
                            "{description} made no progress for {} seconds",
                            idle_timeout.as_secs()
                        ));
                    }
                }
            }
        }

        crate::check_update_cancelled(root)?;
        Ok((received, content_length))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        thread::JoinHandle,
        time::Instant,
    };

    fn stalled_server(send_partial_body: bool) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            read_request_headers(&mut stream);
            if send_partial_body {
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nx",
                );
                let _ = stream.flush();
            }
            let mut byte = [0_u8; 1];
            let _ = stream.read(&mut byte);
        });
        (format!("http://{address}/stalled"), server)
    }

    fn read_request_headers(stream: &mut TcpStream) {
        let mut request = Vec::new();
        let mut buffer = [0_u8; 512];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => return,
                Ok(count) => request.extend_from_slice(&buffer[..count]),
            }
        }
    }

    fn cancellation_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(shroudforge_loader_config::paths::updates_dir(root.path()))
            .unwrap();
        root
    }

    fn request(url: &str) -> reqwest::RequestBuilder {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap()
            .get(url)
    }

    #[test]
    fn cancellation_interrupts_waiting_for_response_headers() {
        let root = cancellation_root();
        let (url, server) = stalled_server(false);
        let cancel_path = crate::check_update_cancelled::cancel_path(root.path());
        let cancellation = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(75));
            std::fs::write(cancel_path, b"cancel").unwrap();
        });
        let started = Instant::now();

        let result = download_response_body(
            root.path(),
            request(&url),
            1024,
            Duration::from_secs(2),
            "test download",
            "too large",
            |_| Ok(()),
            |_, _| Ok(()),
        );

        assert_eq!(result.unwrap_err(), "UPDATE_CANCELLED");
        assert!(started.elapsed() < Duration::from_secs(2));
        cancellation.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn cancellation_interrupts_a_stalled_response_body() {
        let root = cancellation_root();
        let (url, server) = stalled_server(true);
        let cancel_path = crate::check_update_cancelled::cancel_path(root.path());
        let cancellation = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(75));
            std::fs::write(cancel_path, b"cancel").unwrap();
        });
        let started = Instant::now();

        let result = download_response_body(
            root.path(),
            request(&url),
            1024,
            Duration::from_secs(2),
            "test download",
            "too large",
            |_| Ok(()),
            |_, _| Ok(()),
        );

        assert_eq!(result.unwrap_err(), "UPDATE_CANCELLED");
        assert!(started.elapsed() < Duration::from_secs(2));
        cancellation.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn idle_timeout_ends_a_stalled_response_body() {
        let root = cancellation_root();
        let (url, server) = stalled_server(true);
        let started = Instant::now();

        let result = download_response_body(
            root.path(),
            request(&url),
            1024,
            Duration::from_millis(300),
            "test download",
            "too large",
            |_| Ok(()),
            |_, _| Ok(()),
        );

        assert!(result.unwrap_err().contains("made no progress"));
        assert!(started.elapsed() < Duration::from_secs(2));
        server.join().unwrap();
    }
}
