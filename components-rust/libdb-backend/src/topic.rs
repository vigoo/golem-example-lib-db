use crate::catalog::CatalogClient;
use crate::log::Logger;
use crate::topic_discovery::TopicDiscoveryClient;
use crate::LibraryReference;
use golem_rust::{agent_definition, agent_implementation};
use std::collections::HashSet;

/// Agent representing a topic for a library. Discovered libraries are automatically
/// added to the topics the LLM assigns them to. It is possible to search for more libraries
/// for a given topic using web search with an explicit trigger.
#[agent_definition]
pub trait Topic {
    fn new(name: String) -> Self;

    fn discover_libraries(&mut self);

    fn add(&mut self, lib: LibraryReference);

    fn record_failure(&mut self, failure: String);

    fn get_libraries(&self) -> Result<Vec<LibraryReference>, Vec<String>>; // Cannot use HashSet
}

struct TopicImpl {
    name: String,
    libraries: HashSet<LibraryReference>,
    failures: Vec<String>,
    logger: Logger,
}

#[agent_implementation]
impl Topic for TopicImpl {
    fn new(name: String) -> Self {
        if name.to_lowercase() != name {
            panic!("Topic names must be lowercase");
        }

        let mut catalog = CatalogClient::get();
        catalog.trigger_register_topic(name.clone());

        Self {
            logger: Logger::new(&format!("topic {name}")),
            name,
            libraries: HashSet::new(),
            failures: Vec::new(),
        }
    }

    fn discover_libraries(&mut self) {
        let discovery = TopicDiscoveryClient::get(self.name.clone());
        discovery.trigger_run();
    }

    fn add(&mut self, lib: LibraryReference) {
        self.logger.info(format!("Adding library: {lib}"));

        self.libraries.insert(lib);
    }

    fn record_failure(&mut self, failure: String) {
        self.logger
            .error(format!("Topic discovery failed: {failure}"));

        self.failures.push(failure);
    }

    fn get_libraries(&self) -> Result<Vec<LibraryReference>, Vec<String>> {
        if self.failures.is_empty() {
            Ok(self.libraries.iter().cloned().collect())
        } else {
            Err(self.failures.clone())
        }
    }
}
