use crate::*;

#[derive(Clone)]
pub(crate) struct LogWriter(pub(crate) Arc<PathBuf>);

pub(crate) struct Output {
    writer: LogWriter,
    buffer: Vec<u8>,
}

impl Output {
    fn commit(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let bytes = std::mem::take(&mut self.buffer);
        with_log_lock(&self.writer.0, || {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&*self.writer.0)?;
            file.write_all(&bytes)?;
            file.flush()
        })
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        let _ = self.commit();
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogWriter {
    type Writer = Output;

    fn make_writer(&'a self) -> Self::Writer {
        Output {
            writer: self.clone(),
            buffer: Vec::new(),
        }
    }
}

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        io::stdout().write_all(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.commit()?;
        io::stdout().flush()
    }
}
