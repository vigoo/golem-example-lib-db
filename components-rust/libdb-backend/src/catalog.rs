use crate::log::Logger;
use crate::LibraryReference;
use golem_rust::{agent_definition, agent_implementation};
use std::collections::HashSet;

/// A singleton agent where all the discovered libraries and tags are getting registered.
#[agent_definition]
trait Catalog {
    fn new() -> Self;

    fn get_libraries(&self) -> Vec<LibraryReference>;
    fn get_topics(&self) -> Vec<String>;

    fn register_library(&mut self, library: LibraryReference);
    fn register_topic(&mut self, topic: String);
}

struct CatalogImpl {
    libraries: HashSet<LibraryReference>,
    topics: HashSet<String>,
    logger: Logger,
}

#[agent_implementation]
impl Catalog for CatalogImpl {
    fn new() -> Self {
        Self {
            libraries: HashSet::new(),
            topics: HashSet::new(),
            logger: Logger::new("catalog"),
        }
    }

    fn get_libraries(&self) -> Vec<LibraryReference> {
        self.libraries.iter().cloned().collect()
    }

    fn get_topics(&self) -> Vec<String> {
        self.topics.iter().cloned().collect()
    }

    fn register_library(&mut self, library: LibraryReference) {
        self.logger.info(format!("Registering library: {library}"));
        self.libraries.insert(library);
    }

    fn register_topic(&mut self, topic: String) {
        self.logger.info(format!("Registering topic: {topic}"));
        self.topics.insert(topic);
    }
}
