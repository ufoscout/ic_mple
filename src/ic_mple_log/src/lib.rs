use std::sync::Arc;
use std::{cell::RefCell, io};

use arc_swap::{ArcSwap, ArcSwapAny};
use env_filter::{self, Filter};
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

pub mod fmt;
pub mod platform;

use self::fmt::writer::{self, Writer};
use self::fmt::{FormatFn, Formatter};

/// The env logger.
///
/// This struct implements the `Log` trait from the [`log` crate][log-crate-url],
/// which allows it to act as a logger.
///
/// The [`init()`], [`try_init()`], [`Builder::init()`] and [`Builder::try_init()`]
/// methods will each construct a `Logger` and immediately initialize it as the
/// default global logger.
///
/// If you'd instead need access to the constructed `Logger`, you can use
/// the associated [`Builder`] and install it with the
/// [`log` crate][log-crate-url] directly.
///
/// [log-crate-url]: https://docs.rs/log/
/// [`init()`]: fn.init.html
/// [`try_init()`]: fn.try_init.html
/// [`Builder::init()`]: struct.Builder.html#method.init
/// [`Builder::try_init()`]: struct.Builder.html#method.try_init
/// [`Builder`]: struct.Builder.html
pub struct Logger {
    writer: Writer,
    filter: Arc<ArcSwapAny<Arc<Filter>>>,
    format: FormatFn,
}

/// `Builder` acts as builder for initializing a `Logger`.
///
/// It can be used to customize the log format, change the environment variable used
/// to provide the logging directives and also set the default log level filter.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate log;
/// # use std::io::Write;
/// use ic_mple_log::Builder;
/// use log::LevelFilter;
///
/// let mut builder = Builder::new();
///
/// builder
///     .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
///     .filter(None, LevelFilter::Info)
///     .init();
///
/// error!("error message");
/// info!("info message");
/// ```
#[derive(Default)]
pub struct Builder {
    filter: env_filter::Builder,
    writer: writer::Builder,
    format: fmt::Builder,
    built: bool,
}

impl Builder {
    /// Initializes the log builder with defaults.
    ///
    /// **NOTE:** This method won't read from any environment variables.
    /// Use the [`filter`] and [`write_style`] methods to configure the builder
    /// or use [`from_env`] or [`from_default_env`] instead.
    ///
    /// # Examples
    ///
    /// Create a new builder and configure filters and style:
    ///
    /// ```
    /// use log::LevelFilter;
    /// use ic_mple_log::Builder;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder
    ///     .filter(None, LevelFilter::Info)
    ///     .init();
    /// ```
    ///
    /// [`filter`]: #method.filter
    /// [`write_style`]: #method.write_style
    pub fn new() -> Builder {
        Default::default()
    }

    /// Sets the format function for formatting the log output.
    ///
    /// This function is called on each record logged and should format the
    /// log record and output it to the given [`Formatter`].
    ///
    /// The format function is expected to output the string directly to the
    /// `Formatter` so that implementations can use the [`std::fmt`] macros
    /// to format and output without intermediate heap allocations. The default
    /// `ic_mple_log` formatter takes advantage of this.
    ///
    /// # Examples
    ///
    /// Use a custom format to write only the log message:
    ///
    /// ```
    /// use std::io::Write;
    /// use ic_mple_log::Builder;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    /// ```
    ///
    /// [`Formatter`]: fmt/struct.Formatter.html
    /// [`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html
    /// [`std::fmt`]: https://doc.rust-lang.org/std/fmt/index.html
    pub fn format<F>(&mut self, format: F) -> &mut Self
    where
        F: 'static + Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send,
    {
        self.format.custom_format = Some(Box::new(format));
        self
    }

    /// Use the default format.
    ///
    /// This method will clear any custom format set on the builder.
    pub fn default_format(&mut self) -> &mut Self {
        self.format = Default::default();
        self
    }

    /// Whether or not to write the level in the default format.
    pub fn format_level(&mut self, write: bool) -> &mut Self {
        self.format.format_level = write;
        self
    }

    /// Whether or not to write the module path in the default format.
    pub fn format_module_path(&mut self, write: bool) -> &mut Self {
        self.format.format_module_path = write;
        self
    }

    /// Whether or not to write the target in the default format.
    pub fn format_target(&mut self, write: bool) -> &mut Self {
        self.format.format_target = write;
        self
    }

    /// Configures the amount of spaces to use to indent multiline log records.
    /// A value of `None` disables any kind of indentation.
    pub fn format_indent(&mut self, indent: Option<usize>) -> &mut Self {
        self.format.format_indent = indent;
        self
    }

    /// Configures the end of line suffix.
    pub fn format_suffix(&mut self, suffix: &'static str) -> &mut Self {
        self.format.format_suffix = suffix;
        self
    }

    /// Adds a directive to the filter for a specific module.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs in `path::to::module`:
    ///
    /// ```
    /// use ic_mple_log::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter_module("path::to::module", LevelFilter::Info);
    /// ```
    pub fn filter_module(&mut self, module: &str, level: LevelFilter) -> &mut Self {
        self.filter.filter_module(module, level);
        self
    }

    /// Adds a directive to the filter for all modules.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs globally:
    ///
    /// ```
    /// use ic_mple_log::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter_level(LevelFilter::Info);
    /// ```
    pub fn filter_level(&mut self, level: LevelFilter) -> &mut Self {
        self.filter.filter_level(level);
        self
    }

    /// Adds filters to the logger.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs in `path::to::module`:
    ///
    /// ```
    /// use ic_mple_log::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter(Some("path::to::module"), LevelFilter::Info);
    /// ```
    pub fn filter(&mut self, module: Option<&str>, level: LevelFilter) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    /// Parses the directives string in the same form as the `RUST_LOG`
    /// environment variable.
    ///
    /// See the module documentation for more details.
    pub fn parse_filters(&mut self, filters: &str) -> &mut Self {
        self.filter.parse(filters);
        self
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Errors
    ///
    /// This function will fail if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn try_init(&mut self) -> Result<LoggerConfig, SetLoggerError> {
        let (logger, filter) = self.build();

        let max_level = logger.filter();
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(max_level);
        Ok(filter)
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn init(&mut self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    /// Build an env logger.
    ///
    /// The returned logger implements the `Log` trait and can be installed manually
    /// or nested within another logger.
    pub fn build(&mut self) -> (Logger, LoggerConfig) {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        let filter = Arc::new(ArcSwap::from_pointee(self.filter.build()));

        (
            Logger {
                writer: self.writer.build(),
                filter: filter.clone(),
                format: self.format.build(),
            },
            LoggerConfig { filter },
        )
    }
}

pub struct LoggerConfig {
    filter: Arc<ArcSwapAny<Arc<Filter>>>,
}

impl LoggerConfig {
    // Updates the logger filter
    pub fn update_filters(&self, filters: &str) {
        let new_filter = env_filter::Builder::default().parse(filters).build();
        let max_level = new_filter.filter();
        self.filter.swap(Arc::new(new_filter));
        log::set_max_level(max_level);
    }
}

impl Logger {
    /// Returns the maximum `LevelFilter` that this env logger instance is
    /// configured to output.
    pub fn filter(&self) -> LevelFilter {
        self.filter.load().filter()
    }

    /// Checks if this record matches the configured filter.
    pub fn matches(&self, record: &Record) -> bool {
        self.filter.load().matches(record)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.load().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.matches(record) {
            // Log records are written to a thread-local buffer before being printed
            // to the terminal. We clear these buffers afterwards, but they aren't shrunk
            // so will always at least have capacity for the largest log record formatted
            // on that thread.
            //
            // If multiple `Logger`s are used by the same threads then the thread-local
            // formatter might have different color support. If this is the case the
            // formatter and its buffer are discarded and recreated.

            thread_local! {
                static FORMATTER: RefCell<Option<Formatter>> = const { RefCell::new(None) };
            }

            let print = |formatter: &mut Formatter, record: &Record| {
                let _ =
                    (self.format)(formatter, record).and_then(|_| formatter.print(&self.writer));

                // Always clear the buffer afterwards
                formatter.clear();
            };

            let printed = FORMATTER
                .try_with(|tl_buf| {
                    match tl_buf.try_borrow_mut() {
                        // There are no active borrows of the buffer
                        Ok(mut tl_buf) => match *tl_buf {
                            // We have a previously set formatter
                            Some(ref mut formatter) => {
                                print(formatter, record);
                            }
                            // We don't have a previously set formatter
                            None => {
                                let mut formatter = Formatter::new(&self.writer);
                                print(&mut formatter, record);

                                *tl_buf = Some(formatter);
                            }
                        },
                        // There's already an active borrow of the buffer (due to re-entrancy)
                        Err(_) => {
                            print(&mut Formatter::new(&self.writer), record);
                        }
                    }
                })
                .is_ok();

            if !printed {
                // The thread-local storage was not available (because its
                // destructor has already run). Create a new single-use
                // Formatter on the stack for this call.
                print(&mut Formatter::new(&self.writer), record);
            }
        }
    }

    fn flush(&self) {}
}

mod std_fmt_impls {
    use super::*;
    use std::fmt;

    impl fmt::Debug for Logger {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Logger")
                .field("filter", &self.filter)
                .finish()
        }
    }

    impl fmt::Debug for Builder {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.built {
                f.debug_struct("Logger").field("built", &true).finish()
            } else {
                f.debug_struct("Logger")
                    .field("filter", &self.filter)
                    .field("writer", &self.writer)
                    .finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use log::*;

    use super::*;

    #[test]
    fn update_filter_at_runtime() {
        let config = Builder::default()
            .filter_level(LevelFilter::Debug)
            .try_init()
            .unwrap();

        debug!("This one should be printed");
        info!("This one should be printed");

        config.update_filters("error");

        debug!("This one should NOT be printed");
        info!("This one should NOT be printed");

        config.update_filters("info");

        debug!("This one should NOT be printed");
        info!("This one should be printed");
    }
}
