use crate::catalog::CatalogClient;
use crate::log::Logger;
use crate::topic::TopicClient;
use crate::{LibraryDetails, LibraryReference};
use golem_rust::{agent_definition, agent_implementation};
use http::Uri;
use std::collections::HashSet;

enum LibraryState {
    Unknown,
    Analysed {
        repository: Uri,
        description: String,
        topics: HashSet<String>,
    },
    Failed {
        message: String,
    },
}

/// Agent representing a library. Initially it has no other information than
/// its reference (name and language pair), but it gets more details once the analysis completes.
#[agent_definition]
pub trait Library {
    fn new(reference: LibraryReference) -> Self;

    fn analysis_failed(&mut self, message: String);
    async fn analysis_succeeded(
        &mut self,
        repository: Uri,
        description: String,
        topics: Vec<String>,
    );

    fn get_details(&self) -> Result<LibraryDetails, String>;
}

struct LibraryImpl {
    reference: LibraryReference,
    state: LibraryState,
    logger: Logger,
}

#[agent_implementation]
impl Library for LibraryImpl {
    fn new(reference: LibraryReference) -> Self {
        Self {
            logger: Logger::new(&format!("library {reference}")),
            reference,
            state: LibraryState::Unknown,
        }
    }

    fn analysis_failed(&mut self, message: String) {
        self.logger
            .error(format!("Library analysis failed: {message}"));

        self.state = LibraryState::Failed { message };
    }

    async fn analysis_succeeded(
        &mut self,
        repository: Uri,
        description: String,
        topics: Vec<String>,
    ) {
        self.logger.info(format!(
            "Library analysis based on {repository} succeeded with description: {description} and topics: {topics:?}"
        ));

        for topic in &topics {
            let mut topic = TopicClient::get(topic.clone());
            topic.trigger_add(self.reference.clone());
        }

        self.state = LibraryState::Analysed {
            repository,
            description,
            topics: topics.into_iter().collect(),
        };

        let mut catalog = CatalogClient::get();
        catalog.trigger_register_library(self.reference.clone());
    }

    fn get_details(&self) -> Result<LibraryDetails, String> {
        match &self.state {
            LibraryState::Failed { message } => Err(message.clone()),
            LibraryState::Analysed {
                description,
                topics,
                repository,
            } => Ok(LibraryDetails {
                description: description.clone(),
                name: self.reference.name.clone(),
                language: self.reference.language.clone(),
                repository: repository.clone(),
                topics: topics.iter().cloned().collect(),
            }),
            LibraryState::Unknown => Err("Library not yet analysed".to_string()),
        }
    }
}
