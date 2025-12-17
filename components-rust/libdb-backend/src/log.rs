use golem_rust::bindings::wasi::logging::logging::log;
use golem_rust::{agent_definition, agent_implementation, Schema};

#[derive(Schema)]
enum Level {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Log aggregation agent
#[agent_definition]
trait Log {
    fn new() -> Self;
    fn log(&self, level: Level, sender: String, message: String);
}

struct LogImpl {}

#[agent_implementation]
impl Log for LogImpl {
    fn new() -> Self {
        Self {}
    }

    fn log(&self, level: Level, sender: String, message: String) {
        log(level.into(), &sender, &message)
    }
}

impl From<Level> for golem_rust::bindings::wasi::logging::logging::Level {
    fn from(value: Level) -> Self {
        match value {
            Level::Trace => Self::Trace,
            Level::Debug => Self::Debug,
            Level::Info => Self::Info,
            Level::Warning => Self::Warn,
            Level::Error => Self::Error,
            Level::Critical => Self::Critical,
        }
    }
}

/// Logger emits log both to the current agent and also sends them to the singleton Log agent
pub struct Logger {
    client: LogClient,
    sender: String,
}

#[allow(dead_code)]
impl Logger {
    pub fn new(sender: &str) -> Self {
        Self {
            client: LogClient::get(),
            sender: sender.to_string(),
        }
    }

    pub fn trace(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Trace,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Trace,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Debug,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Debug,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }

    pub fn info(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Info,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Info,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Warn,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Warning,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }

    pub fn error(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Error,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Error,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }

    pub fn critical(&self, message: impl AsRef<str>) {
        log(
            golem_rust::bindings::wasi::logging::logging::Level::Critical,
            &self.sender,
            message.as_ref(),
        );
        self.client.trigger_log(
            Level::Critical,
            self.sender.clone(),
            message.as_ref().to_string(),
        );
    }
}
