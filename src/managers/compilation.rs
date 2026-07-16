use std::error::Error;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::model::user::User;

use crate::apis::godbolt::GodboltService;
use crate::apis::wandbox::WandboxService;
use crate::boilerplate::generator::boilerplate_factory;
use crate::utls::constants::JAVA_PUBLIC_CLASS_REGEX;
use crate::utls::discordhelpers::embeds::{EmbedOptions, ToEmbed};
use crate::utls::parser::{shortname_to_qualified, ParserResult};
use godbolt::{CompilationFilters, CompilerOptions, LibrarySelection, PpOptions, RequestOptions};

/// Information about a compilation that callers may need
#[derive(Default, Clone)]
pub struct CompilationDetails {
    /// The language that was compiled (e.g., "c++", "rust", "python")
    pub language: String,
    /// The compiler used (e.g., "g++ 12.1", "rustc 1.65")
    pub compiler: String,
    /// Base64-encoded godbolt link state, if available
    pub godbolt_base64: Option<String>,
    /// Whether compilation/execution succeeded
    pub success: bool,
}

/// The result of a compilation request, containing everything needed to display to the user
pub struct CompilationResult {
    pub details: CompilationDetails,
    pub embed: CreateEmbed,
}

/// Which backend service handles a given language
enum Backend {
    CompilerExplorer,
    WandBox,
}

/// Manages compilation requests across multiple backend services (Godbolt, WandBox).
///
/// Resolution order:
/// 1. Some languages are hardcoded to WandBox (scala, nim, typescript, javascript)
/// 2. If Compiler Explorer supports the target, use it
/// 3. Fall back to WandBox
pub struct CompilationManager {
    wandbox: Option<WandboxService>,
    godbolt: Option<GodboltService>,
}

impl CompilationManager {
    /// Check if a target (language or compiler) is supported by any backend
    pub fn is_target_supported(&self, target: &str) -> bool {
        self.resolve_backend(target).is_some()
    }

    /// Get reference to Godbolt instance, if available
    pub fn godbolt(&self) -> Option<&GodboltService> {
        self.godbolt.as_ref()
    }

    /// Get reference to WandBox instance, if available
    pub fn wandbox(&self) -> Option<&WandboxService> {
        self.wandbox.as_ref()
    }
}

impl CompilationManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let mut broken_compilers = std::collections::HashSet::new();
        broken_compilers.insert(String::from("ghc-head"));
        broken_compilers.insert(String::from("go-head"));

        let mut broken_languages = std::collections::HashSet::new();
        broken_languages.insert(String::from("cpp"));

        let wandbox = match WandboxService::new(broken_compilers, broken_languages).await {
            Ok(wb) => Some(wb),
            Err(e) => {
                error!("Unable to load WandBox: {}", e);
                None
            }
        };

        let godbolt = match GodboltService::new().await {
            Ok(gb) => Some(gb),
            Err(e) => {
                error!("Unable to load Compiler Explorer: {}", e);
                None
            }
        };

        Ok(CompilationManager { wandbox, godbolt })
    }

    /// Compile code and return a result ready for display.
    pub async fn compile(
        &self,
        request: &ParserResult,
        author: &User,
    ) -> Result<CompilationResult, CommandError> {
        match self.resolve_backend(&request.target) {
            Some(Backend::CompilerExplorer) => {
                self.compile_with_godbolt(request, author, false).await
            }
            Some(Backend::WandBox) => self.compile_with_wandbox(request, author).await,
            None => {
                let target = if request.target.starts_with('@') {
                    format!("\\{}", request.target)
                } else {
                    request.target.clone()
                };
                Err(CommandError::from(format!(
                    "Unable to find compiler or language for target '{}'.",
                    target
                )))
            }
        }
    }

    /// Compile code and return assembly output (Godbolt only).
    pub async fn assembly(
        &self,
        request: &ParserResult,
        author: &User,
    ) -> Result<CompilationResult, CommandError> {
        self.compile_with_godbolt(request, author, true).await
    }

    /// Compile using Compiler Explorer (godbolt.org).
    /// Used by both `compile` (execute mode) and `assembly` (asm mode).
    async fn compile_with_godbolt(
        &self,
        request: &ParserResult,
        author: &User,
        asm_mode: bool,
    ) -> Result<CompilationResult, CommandError> {
        let godbolt = self.godbolt.as_ref().ok_or_else(|| {
            CommandError::from(
                "Compiler Explorer is unavailable. This may be due to an outage. Please try again later.",
            )
        })?;

        // Resolve target to a specific compiler
        let target = normalize_target(&request.target);
        let compiler = godbolt.resolve(target).ok_or_else(|| {
            if asm_mode {
                CommandError::from(format!(
                    "Target '{}' either does not produce assembly or is not supported on godbolt.org",
                    target
                ))
            } else {
                CommandError::from(format!(
                    "Unable to find compiler for target '{}'.",
                    target
                ))
            }
        })?;

        // Split any `-lib` requests out of the compiler options
        let mut request = request.clone();
        let library_specs = take_library_specs(&mut request.options)?;
        let request = &request;

        // Prepare code with boilerplate if needed

        let code = if !asm_mode {
            boilerplate_generation(&compiler.lang, &request.code)
        } else {
            request.code.to_owned()
        };

        // Build request options
        let mut options = if asm_mode {
            build_asm_options(request)
        } else {
            build_execute_options(request)
        };
        options.libraries = resolve_libraries(godbolt, &compiler.lang, &library_specs).await?;
        let preprocessor = options.compiler_options.produce_pp.is_some();

        // Get shareable link
        let godbolt_base64 = GodboltService::get_base64(&compiler, &code, &options).ok();

        // Send compilation request
        let response = godbolt.compile(&compiler, &code, options).await?;

        let details = CompilationDetails {
            language: compiler.lang.clone(),
            compiler: compiler.name.clone(),
            godbolt_base64,
            success: response.code == 0,
        };

        let embed_options = EmbedOptions::new(asm_mode, preprocessor, details.clone());
        let embed = response.to_embed(author, &embed_options);

        Ok(CompilationResult { details, embed })
    }

    /// Compile using WandBox.
    async fn compile_with_wandbox(
        &self,
        request: &ParserResult,
        author: &User,
    ) -> Result<CompilationResult, CommandError> {
        let wandbox = self.wandbox.as_ref().ok_or_else(|| {
            CommandError::from(
                "WandBox is unavailable. This may be due to an outage. Please try again later.",
            )
        })?;

        // Resolve target to language and compiler
        let (language, compiler_name) = self.resolve_wandbox_target(wandbox, &request.target)?;

        // Prepare code with boilerplate if needed
        let code = boilerplate_generation(&language, &request.code);

        // Build and send compilation request
        let compilation_request = wandbox::CompilationRequest {
            compiler: compiler_name.clone(),
            code: code.trim().to_string(),
            stdin: request.stdin.trim().to_string(),
            compiler_option_raw: request.options.join("\n"),
            runtime_option_raw: request.args.join("\n"),
            save: false,
            ..Default::default()
        };

        let response = wandbox.compile(&compilation_request).await?;

        let details = CompilationDetails {
            language,
            compiler: compiler_name,
            godbolt_base64: None,
            success: response.status == Some(0),
        };

        let embed_options = EmbedOptions::new(false, false, details.clone());
        let embed = response.to_embed(author, &embed_options);

        Ok(CompilationResult { details, embed })
    }

    /// Directly compile using Compiler Explorer and return raw response.
    /// Used by the ;cpp command which needs access to the raw response.
    pub async fn compile_godbolt_raw(
        &self,
        request: &ParserResult,
    ) -> Result<(CompilationDetails, godbolt::CompilationResult), CommandError> {
        let godbolt = self.godbolt.as_ref().ok_or_else(|| {
            CommandError::from(
                "Compiler Explorer is unavailable. This may be due to an outage. Please try again later.",
            )
        })?;

        let target = normalize_target(&request.target);
        let compiler = godbolt.resolve(target).ok_or_else(|| {
            CommandError::from(format!("Unable to find compiler for target '{}'.", target))
        })?;

        // Split any `-lib` requests out of the compiler options
        let mut request = request.clone();
        let library_specs = take_library_specs(&mut request.options)?;
        let request = &request;

        let code = boilerplate_generation(&compiler.lang, &request.code);
        let mut options = build_execute_options(request);
        options.libraries = resolve_libraries(godbolt, &compiler.lang, &library_specs).await?;
        let godbolt_base64 = GodboltService::get_base64(&compiler, &code, &options).ok();
        let response = godbolt.compile(&compiler, &code, options).await?;

        let details = CompilationDetails {
            language: compiler.lang.clone(),
            compiler: compiler.name.clone(),
            godbolt_base64,
            success: response.code == 0,
        };

        Ok((details, response))
    }

    /// Determine which backend should handle the given target.
    fn resolve_backend(&self, target: &str) -> Option<Backend> {
        // These languages are only available on WandBox
        const WANDBOX_ONLY: &[&str] = &["scala", "nim", "typescript", "javascript"];
        if WANDBOX_ONLY.contains(&target) {
            return Some(Backend::WandBox);
        }

        // Try Compiler Explorer first
        if let Some(ref godbolt) = self.godbolt {
            if godbolt.resolve(target).is_some() {
                return Some(Backend::CompilerExplorer);
            }
        }

        // Fall back to WandBox
        if let Some(ref wandbox) = self.wandbox {
            if wandbox.is_valid_language(target) || wandbox.is_valid_compiler_str(target) {
                return Some(Backend::WandBox);
            }
        }

        None
    }

    /// Resolve a WandBox target to (language, compiler_name).
    fn resolve_wandbox_target(
        &self,
        wandbox: &WandboxService,
        target: &str,
    ) -> Result<(String, String), CommandError> {
        for lang in wandbox.get_languages() {
            // Check if target matches language name
            if target == lang.name {
                let compiler_name = lang
                    .compilers
                    .first()
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                return Ok((lang.name.clone(), compiler_name));
            }

            // Check if target matches a compiler name
            for compiler in &lang.compilers {
                if target == compiler.name {
                    return Ok((lang.name.clone(), compiler.name.clone()));
                }
            }
        }

        Err(CommandError::from(format!(
            "Unable to find language or compiler for target '{}'.",
            target
        )))
    }

    /// Get list of available compilers for a language.
    pub fn get_compiler_list(
        &self,
        language: &str,
        filter: Option<&str>,
    ) -> Result<Vec<String>, CommandError> {
        let lower_lang = language.to_lowercase();
        let language = shortname_to_qualified(&lower_lang);

        match self.resolve_backend(language) {
            Some(Backend::CompilerExplorer) => self.list_godbolt_compilers(language, filter),
            Some(Backend::WandBox) => self.list_wandbox_compilers(language, filter),
            None => Err(CommandError::from(format!(
                "Unable to find compilers for target '{}'.",
                language
            ))),
        }
    }

    /// Get list of available Compiler Explorer libraries for a language.
    pub async fn get_library_list(
        &self,
        language: &str,
        filter: Option<&str>,
    ) -> Result<Vec<String>, CommandError> {
        let lower_lang = language.to_lowercase();
        let language = shortname_to_qualified(&lower_lang);

        let godbolt = self.godbolt.as_ref().ok_or_else(|| {
            CommandError::from(
                "Compiler Explorer is unavailable. This may be due to an outage. Please try again later.",
            )
        })?;

        if !godbolt
            .cache
            .iter()
            .any(|entry| entry.language.id == language)
        {
            return Err(CommandError::from(format!(
                "Unable to find language '{}'. Libraries are only available for Compiler Explorer targets.",
                language
            )));
        }

        let libraries = godbolt.libraries_for(language).await.map_err(|e| {
            CommandError::from(format!(
                "Unable to fetch libraries for '{}': {}",
                language, e
            ))
        })?;

        let mut results: Vec<(f64, String)> = Vec::new();

        for library in libraries {
            let versions = match (library.versions.first(), library.versions.last()) {
                (Some(first), Some(last)) if library.versions.len() > 1 => {
                    format!("{} … {}", first.version, last.version)
                }
                (Some(only), _) => only.version.clone(),
                _ => String::from("none"),
            };
            let display = format!("{} -> **{}** ({})", library.name, library.id, versions);

            if let Some(filter_str) = filter {
                if !matches_filter(&library.id, &library.name, filter_str) {
                    continue;
                }
                let similarity = compute_similarity(&library.id, &library.name, filter_str);
                results.push((similarity, display));
            } else {
                results.push((0.0, display));
            }
        }

        if filter.is_some() {
            results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        }

        Ok(results.into_iter().map(|(_, s)| s).collect())
    }

    fn list_godbolt_compilers(
        &self,
        language: &str,
        filter: Option<&str>,
    ) -> Result<Vec<String>, CommandError> {
        let godbolt = self.godbolt.as_ref().unwrap();
        let mut results: Vec<(f64, String)> = Vec::new();

        for cache_entry in &godbolt.cache {
            if cache_entry.language.id != language {
                continue;
            }

            for compiler in &cache_entry.compilers {
                let display = format!("{} -> **{}**", &compiler.name, &compiler.id);

                if let Some(filter_str) = filter {
                    if !matches_filter(&compiler.id, &compiler.name, filter_str) {
                        continue;
                    }
                    let similarity = compute_similarity(&compiler.id, &compiler.name, filter_str);
                    results.push((similarity, display));
                } else {
                    results.push((0.0, display));
                }
            }
        }

        if filter.is_some() {
            results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        }

        Ok(results.into_iter().map(|(_, s)| s).collect())
    }

    fn list_wandbox_compilers(
        &self,
        language: &str,
        filter: Option<&str>,
    ) -> Result<Vec<String>, CommandError> {
        let wandbox = self.wandbox.as_ref().unwrap();
        let compilers = wandbox
            .get_compilers(shortname_to_qualified(language))
            .ok_or_else(|| {
                CommandError::from(format!(
                    "Unable to find compilers for target '{}'.",
                    language
                ))
            })?;

        let mut results: Vec<(f64, String)> = Vec::new();

        for compiler in compilers {
            if let Some(filter_str) = filter {
                if !matches_filter(&compiler.name, &compiler.name, filter_str) {
                    continue;
                }
                let similarity = similar_string::compare_similarity(filter_str, &compiler.name);
                results.push((similarity, compiler.name));
            } else {
                results.push((0.0, compiler.name));
            }
        }

        if filter.is_some() {
            results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        }

        Ok(results.into_iter().map(|(_, s)| s).collect())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Normalize target names (e.g., "haskell" -> "ghc901")
fn normalize_target(target: &str) -> &str {
    match target {
        "haskell" => "ghc901",
        other => other,
    }
}

/// Prepare code by adding boilerplate and fixing common issues
fn boilerplate_generation(language: &str, code: &str) -> String {
    let generator = boilerplate_factory(language, code);
    let code = if generator.needs_boilerplate() {
        generator.generate()
    } else {
        code.to_string()
    };

    fix_common_problems(language, code)
}

/// Fix common language-specific issues in user code
fn fix_common_problems(language: &str, code: String) -> String {
    match language {
        "java" => {
            // Remove 'public' from class declarations (Godbolt doesn't like public classes)
            let mut fixed = code.clone();
            for m in JAVA_PUBLIC_CLASS_REGEX.captures_iter(&code) {
                if let Some(pub_keyword) = m.name("public") {
                    fixed.replace_range(pub_keyword.range(), "");
                }
            }
            fixed
        }
        _ => code,
    }
}

/// Remove `-lib <library>:<version>` flags from the compiler options,
/// returning the extracted library specs
fn take_library_specs(options: &mut Vec<String>) -> Result<Vec<String>, CommandError> {
    let mut specs = Vec::new();
    let mut remaining = Vec::new();

    let mut iter = std::mem::take(options).into_iter();
    while let Some(opt) = iter.next() {
        if opt == "-lib" {
            let spec = iter.next().ok_or_else(|| {
                CommandError::from("The `-lib` flag requires a library, e.g. `-lib fmt:trunk`")
            })?;
            specs.push(spec);
        } else {
            remaining.push(opt);
        }
    }

    *options = remaining;
    Ok(specs)
}

/// Resolve `-lib` specs (`<library>:<version>`) against Compiler Explorer's
/// library list for the given language
async fn resolve_libraries(
    godbolt: &GodboltService,
    language_id: &str,
    specs: &[String],
) -> Result<Vec<LibrarySelection>, CommandError> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }

    let libraries = godbolt.libraries_for(language_id).await.map_err(|e| {
        CommandError::from(format!(
            "Unable to fetch libraries for '{}': {}",
            language_id, e
        ))
    })?;

    let mut selections = Vec::new();
    for spec in specs {
        let (name, version_str) = spec.split_once(':').unwrap_or((spec.as_str(), ""));

        let library = libraries
            .iter()
            .find(|lib| lib.id.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                CommandError::from(format!(
                    "Unknown library '{}' for language '{}'. See `;libraries {}` to browse.\n\nAvailable libraries: {}",
                    name,
                    language_id,
                    language_id,
                    summarize_names(libraries.iter().map(|lib| lib.id.as_str()))
                ))
            })?;

        let version = library
            .versions
            .iter()
            .find(|ver| {
                ver.id.eq_ignore_ascii_case(version_str)
                    || ver.version.eq_ignore_ascii_case(version_str)
                    || ver
                        .alias
                        .iter()
                        .any(|alias| alias.eq_ignore_ascii_case(version_str))
            })
            .ok_or_else(|| {
                let versions =
                    summarize_names(library.versions.iter().map(|ver| ver.version.as_str()));
                if version_str.is_empty() {
                    CommandError::from(format!(
                        "Library '{}' requires a version, e.g. `-lib {}:<version>`.\n\nAvailable versions: {}",
                        library.id, library.id, versions
                    ))
                } else {
                    CommandError::from(format!(
                        "Unknown version '{}' for library '{}'.\n\nAvailable versions: {}",
                        version_str, library.id, versions
                    ))
                }
            })?;

        selections.push(LibrarySelection {
            id: library.id.clone(),
            version: version.id.clone(),
        });
    }

    Ok(selections)
}

/// Join names with commas, truncating once the list grows too long for an
/// error message
fn summarize_names<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let mut summary = String::new();
    for name in names {
        if !summary.is_empty() {
            summary.push_str(", ");
        }
        if summary.len() + name.len() > 800 {
            summary.push('…');
            break;
        }
        summary.push_str(name);
    }
    summary
}

/// Build request options for code execution
fn build_execute_options(request: &ParserResult) -> RequestOptions {
    RequestOptions {
        user_arguments: request.options.join(" "),
        compiler_options: CompilerOptions {
            skip_asm: true,
            executor_request: true,
            ..Default::default()
        },
        execute_parameters: godbolt::ExecuteParameters {
            args: request.args.clone(),
            stdin: request.stdin.clone(),
        },
        filters: CompilationFilters {
            comment_only: Some(true),
            demangle: Some(true),
            directives: Some(true),
            execute: Some(true),
            intel: Some(true),
            labels: Some(true),
            trim: Some(true),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Build request options for assembly output
fn build_asm_options(request: &ParserResult) -> RequestOptions {
    // Only request preprocessor output when the user explicitly asks for it via
    // `-E`. When present, we show the cleaner (header-filtered, formatted)
    // preprocessor source instead of the assembly.
    let produce_pp = request
        .options
        .iter()
        .any(|opt| opt == "-E")
        .then_some(PpOptions {
            filter_headers: true,
            clang_format: true,
        });

    RequestOptions {
        user_arguments: request.options.join(" "),
        compiler_options: CompilerOptions {
            skip_asm: false,
            executor_request: false,
            produce_pp,
            ..Default::default()
        },
        execute_parameters: Default::default(),
        filters: CompilationFilters {
            comment_only: Some(true),
            demangle: Some(true),
            directives: Some(true),
            execute: Some(false),
            intel: Some(true),
            labels: Some(true),
            trim: Some(true),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::take_library_specs;

    #[test]
    fn takes_library_specs_out_of_options() {
        let mut options = vec![
            String::from("-O2"),
            String::from("-lib"),
            String::from("fmt:trunk"),
            String::from("-Wall"),
            String::from("-lib"),
            String::from("ctre:dev"),
        ];

        let specs = take_library_specs(&mut options).unwrap();
        assert_eq!(specs, vec!["fmt:trunk", "ctre:dev"]);
        assert_eq!(options, vec!["-O2", "-Wall"]);
    }

    #[test]
    fn leaves_options_without_libraries_alone() {
        let mut options = vec![String::from("-O2"), String::from("-Wall")];

        let specs = take_library_specs(&mut options).unwrap();
        assert!(specs.is_empty());
        assert_eq!(options, vec!["-O2", "-Wall"]);
    }

    #[test]
    fn rejects_dangling_lib_flag() {
        let mut options = vec![String::from("-O2"), String::from("-lib")];
        assert!(take_library_specs(&mut options).is_err());
    }
}

/// Check if a compiler matches the filter string
fn matches_filter(id: &str, name: &str, filter: &str) -> bool {
    filter
        .split_whitespace()
        .any(|part| id.contains(part) || name.contains(part))
}

/// Compute similarity score for sorting filtered results
fn compute_similarity(id: &str, name: &str, filter: &str) -> f64 {
    let id_sim = similar_string::compare_similarity(filter, id);
    let name_sim = similar_string::compare_similarity(filter, name);
    f64::max(id_sim, name_sim)
}
