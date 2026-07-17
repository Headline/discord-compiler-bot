//! Compiler Explorer (godbolt.org) service built on the general-purpose
//! `godbolt` crate.
//!
//! The crate's client performs no I/O at construction and keeps no caches;
//! this service fetches the language/compiler catalog and formatter list once
//! at startup so the rest of the bot can keep resolving targets offline.

use godbolt::{
    AsmDocumentation, ClientState, CompilationRequest, CompilationResult, Compiler, Executor,
    ExecutorCompiler, Format, FormatRequest, FormatResult, Godbolt, GodboltError, Language,
    Library, RequestOptions, Session, SessionCompiler,
};

use crate::utls::constants::USER_AGENT;

/// A language and its compilers, cached at startup
pub struct GodboltCacheEntry {
    pub language: Language,
    pub compilers: Vec<Compiler>,
}

/// Compiler Explorer client plus the startup caches used for offline lookups
pub struct GodboltService {
    client: Godbolt,
    /// Cache of godbolt languages and their associated compilers
    pub cache: Vec<GodboltCacheEntry>,
    /// Cache of all formatting tools
    pub formats: Vec<Format>,
}

impl GodboltService {
    pub async fn new(http: reqwest::Client) -> Result<Self, GodboltError> {
        let client = Godbolt::builder()
            .user_agent(USER_AGENT)
            .http_client(http)
            .build();
        let catalog = client.catalog().await?;
        let formats = client.formats().await?;

        let cache = catalog
            .languages
            .into_iter()
            .map(|language| {
                let compilers = catalog
                    .compilers
                    .iter()
                    .filter(|compiler| compiler.lang == language.id)
                    .cloned()
                    .collect();
                GodboltCacheEntry {
                    language,
                    compilers,
                }
            })
            .collect();

        Ok(GodboltService {
            client,
            cache,
            formats,
        })
    }

    /// Resolves a target to a compiler: first as a compiler id, then as a
    /// language id (yielding that language's default compiler)
    pub fn resolve(&self, target: &str) -> Option<Compiler> {
        if let Some(compiler) = self.find_compiler_by_id(target) {
            return Some(compiler.clone());
        }

        let language = self.find_language_by_id(target)?;
        Some(
            self.find_compiler_by_id(&language.default_compiler)?
                .clone(),
        )
    }

    fn find_compiler_by_id(&self, compiler_id: &str) -> Option<&Compiler> {
        self.cache
            .iter()
            .flat_map(|entry| &entry.compilers)
            .find(|compiler| compiler.id.eq_ignore_ascii_case(compiler_id))
    }

    fn find_language_by_id(&self, language_id: &str) -> Option<&Language> {
        self.cache
            .iter()
            .map(|entry| &entry.language)
            .find(|language| language.id.eq_ignore_ascii_case(language_id))
    }

    /// Fetches the libraries available for a language
    pub async fn libraries_for(&self, language_id: &str) -> Result<Vec<Library>, GodboltError> {
        self.client.libraries_for(language_id).await
    }

    /// Fetches documentation for an assembly opcode
    pub async fn asm_doc(
        &self,
        instruction_set: &str,
        opcode: &str,
    ) -> Result<AsmDocumentation, GodboltError> {
        self.client.asm_doc(instruction_set, opcode).await
    }

    /// Compiles `source` with the given compiler and options
    pub async fn compile(
        &self,
        compiler: &Compiler,
        source: &str,
        options: RequestOptions,
    ) -> Result<CompilationResult, GodboltError> {
        let mut request = CompilationRequest::new(source);
        request.options = options;
        self.client.compile(&compiler.id, &request).await
    }

    /// Builds the base64-encoded client state used for godbolt.org
    /// `/clientstate/` share links
    pub fn get_base64(
        compiler: &Compiler,
        source: &str,
        options: &RequestOptions,
    ) -> Result<String, GodboltError> {
        let state = ClientState {
            sessions: vec![Session {
                id: 0,
                language: compiler.lang.clone(),
                source: source.to_string(),
                compilers: vec![SessionCompiler {
                    id: compiler.id.clone(),
                    options: options.user_arguments.clone(),
                }],
                executors: vec![Executor {
                    arguments: options.execute_parameters.args.join(" "),
                    compiler: ExecutorCompiler {
                        id: compiler.id.clone(),
                        libs: Vec::new(),
                        options: options.user_arguments.clone(),
                    },
                    stdin: options.execute_parameters.stdin.clone(),
                }],
            }],
        };

        state.to_base64()
    }

    /// Formats `source` with the given formatter; an empty `style` means the
    /// formatter's default
    pub async fn format_code(
        &self,
        formatter: &str,
        style: &str,
        source: &str,
        use_spaces: bool,
        tab_width: i32,
    ) -> Result<FormatResult, GodboltError> {
        let mut request = FormatRequest::new(source)
            .use_spaces(use_spaces)
            .tab_width(tab_width);
        if !style.is_empty() {
            request = request.base(style);
        }

        self.client.format(formatter, &request).await
    }
}
