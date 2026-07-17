//! WandBox service built on the general-purpose `wandbox` crate.
//!
//! The crate's client performs no I/O at construction and keeps no caches;
//! this service fetches the compiler list once at startup, grouped by
//! language, so the rest of the bot can keep resolving targets offline.

use std::collections::HashSet;

use wandbox::{CompilationRequest, CompilationResult, Compiler, Wandbox, WandboxError};

use crate::utls::constants::USER_AGENT;

/// A language and its compilers, cached at startup. Language names are
/// lowercased; compilers keep the order wandbox lists them in (newest first)
pub struct WandboxLanguage {
    pub name: String,
    pub compilers: Vec<Compiler>,
}

/// WandBox client plus the startup cache used for offline lookups
pub struct WandboxService {
    client: Wandbox,
    languages: Vec<WandboxLanguage>,
}

impl WandboxService {
    /// Initializes the service, ignoring any compiler names in
    /// `broken_compilers` and any (lowercase) language names in
    /// `broken_languages`
    pub async fn new(
        http: reqwest::Client,
        broken_compilers: HashSet<String>,
        broken_languages: HashSet<String>,
    ) -> Result<Self, WandboxError> {
        let client = Wandbox::builder()
            .user_agent(USER_AGENT)
            .http_client(http)
            .build();

        let mut languages: Vec<WandboxLanguage> = Vec::new();
        for mut compiler in client.compilers().await? {
            compiler.language = compiler.language.to_ascii_lowercase();
            if broken_languages.contains(&compiler.language)
                || broken_compilers.contains(&compiler.name)
            {
                continue;
            }

            match languages
                .iter_mut()
                .find(|lang| lang.name == compiler.language)
            {
                Some(language) => language.compilers.push(compiler),
                None => languages.push(WandboxLanguage {
                    name: compiler.language.clone(),
                    compilers: vec![compiler],
                }),
            }
        }

        Ok(WandboxService { client, languages })
    }

    /// Returns every cached language
    pub fn get_languages(&self) -> &[WandboxLanguage] {
        &self.languages
    }

    /// Determines if the supplied string is a valid language name
    pub fn is_valid_language(&self, language: &str) -> bool {
        self.languages.iter().any(|lang| lang.name == language)
    }

    /// Determines if the supplied string is a valid compiler name
    pub fn is_valid_compiler_str(&self, compiler: &str) -> bool {
        self.languages
            .iter()
            .flat_map(|lang| &lang.compilers)
            .any(|c| c.name == compiler)
    }

    /// Gets the list of compilers for a given language
    pub fn get_compilers(&self, language: &str) -> Option<Vec<Compiler>> {
        self.languages
            .iter()
            .find(|lang| lang.name == language)
            .map(|lang| lang.compilers.clone())
    }

    /// Compiles and runs the given request
    pub async fn compile(
        &self,
        request: &CompilationRequest,
    ) -> Result<CompilationResult, WandboxError> {
        self.client.compile(request).await
    }
}
