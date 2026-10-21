//! Standard output for results and standard error for diagnostics.

use std::io::{self, Write};

/// The two streams of one CLI run, injected so that tests capture them in memory.
pub(crate) struct Terminal<'a> {
    out: &'a mut (dyn Write + Send),
    err: &'a mut (dyn Write + Send),
}

impl<'a> Terminal<'a> {
    /// Selects the result stream and the diagnostic stream.
    pub(crate) fn new(out: &'a mut (dyn Write + Send), err: &'a mut (dyn Write + Send)) -> Self {
        Self { out, err }
    }

    /// Writes one complete result and a final newline to standard output.
    pub(crate) fn result(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.out.write_all(bytes)?;
        self.out.write_all(b"\n")?;
        self.out.flush()
    }

    /// Writes text, such as help, to standard output.
    pub(crate) fn text(&mut self, text: &str) -> io::Result<()> {
        self.out.write_all(text.as_bytes())?;
        self.out.flush()
    }

    /// Writes text, such as a usage error, to standard error.
    pub(crate) fn diagnostic_text(&mut self, text: &str) {
        let _ = self.err.write_all(text.as_bytes());
        let _ = self.err.flush();
    }

    /// Writes one labelled line, such as `error: ...`, to standard error.
    pub(crate) fn diagnostic(&mut self, label: &str, message: &str) {
        let _ = writeln!(self.err, "{label}: {message}");
        let _ = self.err.flush();
    }
}
