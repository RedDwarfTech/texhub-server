use log::{Log, Metadata, Record};

use crate::common::request_context;

pub struct RequestIdLogger {
    inner: log4rs::Logger,
}

impl RequestIdLogger {
    pub fn new(inner: log4rs::Logger) -> Self {
        Self { inner }
    }
}

impl Log for RequestIdLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(request_id) = request_context::try_get_request_id() {
            let enriched = Record::builder()
                .level(record.level())
                .target(record.target())
                .file(record.file())
                .line(record.line())
                .module_path(record.module_path())
                .args(format_args!(
                    "[x-request-id={}] {}",
                    request_id,
                    record.args()
                ))
                .build();
            self.inner.log(&enriched);
        } else {
            self.inner.log(record);
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

pub fn init_logging() -> Result<log4rs::Handle, log::SetLoggerError> {
    let config = log4rs::load_config_file("log4rs.yaml", Default::default())
        .expect("failed to load log4rs.yaml");
    let inner = log4rs::Logger::new(config);
    log::set_max_level(inner.max_log_level());
    let handle = inner.handle();
    log::set_boxed_logger(Box::new(RequestIdLogger::new(inner)))?;
    Ok(handle)
}
