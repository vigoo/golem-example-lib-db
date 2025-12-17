use crate::catalog::CatalogClient;
use crate::library_analysis::LibraryAnalysisClient;
use crate::log::Logger;
use crate::topic::TopicClient;
use crate::LibraryReference;
use golem_rust::{agent_definition, agent_implementation};
use std::collections::HashSet;

/// Agent representing a library. Initially it has no other information than
/// its reference, but it gets more details once the analysis completes.
#[agent_definition]
pub trait Library {
    fn new(reference: LibraryReference) -> Self;

    fn analysis_failed(&mut self, message: String);
    async fn analysis_succeeded(&mut self, description: String, tags: Vec<String>);

    fn get_details(&self) -> Result<(String, Vec<String>), String>;
}

struct LibraryImpl {
    reference: LibraryReference,
    failure: Option<String>,
    description: String,
    topics: HashSet<String>,
    logger: Logger,
}

#[agent_implementation]
impl Library for LibraryImpl {
    fn new(reference: LibraryReference) -> Self {
        let mut analysis = LibraryAnalysisClient::get(reference.clone());
        analysis.trigger_run(None);
        Self {
            logger: Logger::new(&format!("library {reference}")),
            reference,
            failure: None,
            description: "".to_string(),
            topics: HashSet::new(),
        }
    }

    fn analysis_failed(&mut self, message: String) {
        self.logger
            .error(format!("Library analysis failed: {message}"));

        self.failure = Some(message);
    }

    async fn analysis_succeeded(&mut self, description: String, tags: Vec<String>) {
        self.logger.info(format!(
            "Library analysis succeeded with description: {description} and tags: {tags:?}"
        ));

        self.description = description;
        self.topics = tags.into_iter().collect();

        let mut catalog = CatalogClient::get();
        catalog.trigger_register_library(self.reference.clone());

        for topic in &self.topics {
            let mut topic = TopicClient::get(topic.clone());
            topic.trigger_add(self.reference.clone());
        }
    }

    fn get_details(&self) -> Result<(String, Vec<String>), String> {
        match &self.failure {
            Some(message) => Err(message.clone()),
            None => Ok((
                self.description.clone(),
                self.topics.iter().cloned().collect(),
            )),
        }
    }
}
