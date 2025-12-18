mod catalog;
mod library;
mod library_analysis;
mod log;
mod topic;
mod topic_discovery;

use crate::log::Logger;
use golem_rust::{agent_definition, agent_implementation, Schema};
use http::Uri;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Schema)]
pub enum Language {
    Rust,
    JavaScript,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Schema)]
pub struct LibraryReference {
    name: String,
    language: Language,
}

impl Display for LibraryReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{:?}]", self.name, self.language)
    }
}

#[derive(Debug, Clone, Schema)]
pub struct LibraryDetails {
    name: String,
    language: Language,
    repository: Uri,
    description: String,
    topics: HashSet<String>,
}

#[agent_definition]
trait Test {
    fn new() -> Self;
    fn run(&self);
}

struct TestImpl {
    logger: Logger,
}

#[agent_implementation]
impl Test for TestImpl {
    fn new() -> Self {
        Self {
            logger: Logger::new("test"),
        }
    }

    fn run(&self) {
        self.logger.info("Hello, world!");
    }
}
