use crate::library::LibraryClient;
use crate::log::Logger;
use crate::LibraryReference;
use golem_rust::golem_ai::golem::llm::llm::{send, Config, ContentPart, Event, Message, Role};
use golem_rust::{agent_definition, agent_implementation};

/// Background job for each Library agent, triggered on creation. It uses LLM to
/// gather more information about the library.
#[agent_definition]
pub trait LibraryAnalysis {
    fn new(reference: LibraryReference) -> Self;

    async fn run(&mut self, parent_tag: Option<String>);
}

struct LibraryAnalysisImpl {
    reference: LibraryReference,
    logger: Logger,
}

#[agent_implementation]
impl LibraryAnalysis for LibraryAnalysisImpl {
    fn new(reference: LibraryReference) -> Self {
        Self {
            logger: Logger::new(&format!("library analysis {reference}")),
            reference,
        }
    }

    async fn run(&mut self, parent_tag: Option<String>) {
        let response = send(&[Event::Message(
            Message {
                role: Role::User,
                name: None,
                content: vec![
                    ContentPart::Text(format!("Let's analyse the GitHub repository at {}. First check if this is a library for {:?}. If it is, then come up with a list of tags describing what this library is for, and return it as a JSON array of strings. If it is not for the given language, return an empty tag array.", self.reference.repository, self.reference.language)),
                    ContentPart::Text("In addition to the array of tags, also return a short description of the library in a separate field of the result JSON object.".to_string()),
                    ContentPart::Text("Always response with a JSON object with the following structure: { \"description\": \"short description of the library\", \"tags\": [\"tag1\", \"tag2\", ...] }".to_string()),
                ],
            }
        )],
                            &Config {
                                model: "gpt-3.5-turbo".to_string(),
                                temperature: None,
                                max_tokens: None,
                                stop_sequences: None,
                                tools: None,
                                tool_choice: None,
                                provider_options: None,
                            },
        );

        let mut library = LibraryClient::get(self.reference.clone());
        match response {
            Ok(response) => {
                let raw_string_content = response
                    .content
                    .iter()
                    .map(|c| match c {
                        ContentPart::Text(s) => s.clone(),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join("");

                self.logger
                    .debug(format!("LLM response: {raw_string_content}"));

                match serde_json::from_str::<ExpectedLlmResponse>(&raw_string_content) {
                    Ok(response) => {
                        if response.tags.is_empty() {
                            library
                                .analysis_failed(
                                    "Library does not seem to be using the associated language"
                                        .to_string(),
                                )
                                .await;
                        } else {
                            let mut final_tags = response
                                .tags
                                .into_iter()
                                .map(|t| t.to_lowercase())
                                .collect::<Vec<String>>();
                            if let Some(parent_tag) = parent_tag {
                                final_tags.push(parent_tag);
                            }
                            library
                                .analysis_succeeded(response.description, final_tags)
                                .await;
                        }
                    }
                    Err(err) => {
                        self.logger
                            .error(format!("Failed to parse LLM response: {err:?}"));

                        library.analysis_failed(err.to_string()).await;
                    }
                }
            }
            Err(err) => {
                self.logger
                    .error(format!("Failed to analyse library: {err:?}"));
                library.analysis_failed(err.to_string()).await;
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ExpectedLlmResponse {
    description: String,
    tags: Vec<String>,
}
