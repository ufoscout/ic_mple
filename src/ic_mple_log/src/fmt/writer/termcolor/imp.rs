use std::io;

use crate::platform;

pub(in crate::fmt::writer) struct BufferWriter {}

pub(in crate::fmt) struct Buffer(Vec<u8>);

impl BufferWriter {
    pub(in crate::fmt::writer) fn new() -> Self {
        Self {}
    }

    pub(in crate::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in crate::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        platform::print(&buf.0);
        Ok(())
    }
}

impl Buffer {
    pub(in crate::fmt) fn clear(&mut self) {
        self.0.clear();
    }

    pub(in crate::fmt) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }

    pub(in crate::fmt) fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::fmt) fn bytes(&self) -> &[u8] {
        &self.0
    }
}
