mod termcolor;

use self::termcolor::BufferWriter;
use std::{fmt, io};

pub(super) use self::termcolor::Buffer;

/// A terminal target with color awareness.
pub(crate) struct Writer {
    inner: BufferWriter,
}

impl Writer {
    pub(super) fn buffer(&self) -> Buffer {
        self.inner.buffer()
    }

    pub(super) fn print(&self, buf: &Buffer) -> io::Result<()> {
        self.inner.print(buf)
    }
}

/// A builder for a terminal writer.
///
/// The target and style choice can be configured before building.
#[derive(Debug)]
pub(crate) struct Builder {
    built: bool,
}

impl Builder {
    /// Initialize the writer builder with defaults.
    pub(crate) fn new() -> Self {
        Builder { built: false }
    }

    /// Build a terminal writer.
    pub(crate) fn build(&mut self) -> Writer {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;
        Writer {
            inner: BufferWriter::new(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Writer").finish()
    }
}
