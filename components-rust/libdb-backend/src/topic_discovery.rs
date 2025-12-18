use crate::library_analysis::LibraryAnalysisClient;
use crate::log::Logger;
use crate::topic::TopicClient;
use crate::{Language, LibraryReference};
use golem_rust::golem_ai::golem::web_search::types::{SearchParams, SearchResult};
use golem_rust::golem_ai::golem::web_search::web_search::start_search;
use golem_rust::{agent_definition, agent_implementation};
use http::uri::Scheme;
use http::Uri;
use std::str::FromStr;

/// Background job for each Topic agent, only created on an explicit discover_libraries() call.
/// It uses a web search API to look for libraries for various programming languages and a given
/// topic. Each library is then registered as a library, but only gets added to the topic if
/// the library analysis classifies it as relevant.
#[agent_definition]
pub trait TopicDiscovery {
    fn new(name: String) -> Self;

    async fn run(&self);
}

struct TopicDiscoveryImpl {
    name: String,
    logger: Logger,
}

impl TopicDiscoveryImpl {
    fn try_run(&self) -> anyhow::Result<Vec<(Language, SearchResult)>> {
        const LANGUAGES: &[Language] = &[Language::Rust, Language::JavaScript];
        let mut result = vec![];

        for language in LANGUAGES {
            self.logger
                .debug(format!("Searching for libraries in {language:?}..."));

            let search = start_search(&SearchParams {
                query: format!("{} library for {:?}", self.name.clone(), language),
                safe_search: None,
                language: None,
                region: None,
                max_results: None,
                time_range: None,
                include_domains: Some(vec!["github.com".to_string()]),
                exclude_domains: None,
                include_images: Some(false),
                include_html: None,
                advanced_answer: None,
            })?;

            loop {
                let page = search.next_page()?;
                if page.is_empty() {
                    break;
                }

                self.logger
                    .trace(format!("Got page with {} results", page.len()));

                result.extend(page.into_iter().map(|r| (language.clone(), r)));
            }
        }

        Ok(result)
    }
}

#[agent_implementation]
impl TopicDiscovery for TopicDiscoveryImpl {
    fn new(name: String) -> Self {
        Self {
            logger: Logger::new(&format!("topic discovery {name}")),
            name,
        }
    }

    async fn run(&self) {
        match self.try_run() {
            Ok(results) => {
                for (language, result) in results {
                    self.logger.debug(format!(
                        "Hit for {}: {language:?} => {}",
                        self.name, result.url
                    ));
                    let mut library_analysis = LibraryAnalysisClient::get(LibraryReference {
                        name: extract_github_repo_name(result.url.clone()),
                        language,
                        repository: extract_github_repo(result.url),
                    });
                    library_analysis.run(Some(self.name.clone())).await;
                }
            }
            Err(err) => {
                let mut topic = TopicClient::get(self.name.clone());
                topic.record_failure(err.to_string()).await;
            }
        }
    }
}

fn extract_github_repo_name(url: String) -> String {
    url.strip_prefix("https://github.com/")
        .and_then(|path| path.split('/').nth(1))
        .unwrap_or("")
        .to_string()
}

fn extract_github_repo(url: String) -> Uri {
    try_extract_github_repo(url).unwrap_or_default()
}

fn try_extract_github_repo(url: String) -> Option<Uri> {
    let url = Uri::from_str(&url).ok()?;
    if url.host() == Some("github.com") {
        let path = url.path();
        let owner = path.split('/').next()?;
        let repo = path.split('/').nth(1)?;
        Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority("github.com")
            .path_and_query(format!("/{}/{}", owner, repo))
            .build()
            .ok()
    } else {
        None
    }
}
