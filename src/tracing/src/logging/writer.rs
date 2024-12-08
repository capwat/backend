use capwat_config::logging::ConsoleStream;
use std::io::{self, IsTerminal, Write};
use tracing_subscriber::fmt::{MakeWriter, TestWriter};

/// Implements [`tracing_subscriber::fmt::MakeWriter`] with
/// [`LoggingStream`] enum wrapped inside.
pub struct LoggingStreamMaker(ConsoleStream);

impl LoggingStreamMaker {
    #[must_use]
    pub fn new(variant: ConsoleStream) -> Self {
        Self(variant)
    }

    pub fn supports_color(&self) -> bool {
        match self.0 {
            ConsoleStream::Stdout => io::stdout().is_terminal(),
            ConsoleStream::Stderr => io::stderr().is_terminal(),
            ConsoleStream::TestWriter => false,
        }
    }
}

impl<'writer> MakeWriter<'writer> for LoggingStreamMaker {
    type Writer = BoxedWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        match self.0 {
            ConsoleStream::Stdout => BoxedWriter::new(io::stdout()),
            ConsoleStream::Stderr => BoxedWriter::new(io::stderr()),
            ConsoleStream::TestWriter => BoxedWriter::new(TestWriter::new()),
        }
    }
}

/// Represents a writer that wraps any object that implements [`Write`].
pub struct BoxedWriter(Box<dyn Write>);

impl BoxedWriter {
    #[must_use]
    fn new(write: impl Write + 'static) -> Self {
        Self(Box::new(write))
    }
}

impl Write for BoxedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
