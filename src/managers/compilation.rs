use std::error::Error;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::model::user::User;

use crate::boilerplate::generator::boilerplate_factory;
use godbolt::{CompilationFilters, CompilerOptions, Godbolt, GodboltError, RequestOptions};
use wandbox::{CompilationBuilder, Wandbox, WandboxError};

use crate::utls::constants::USER_AGENT;
use crate::utls::discordhelpers::embeds::ToEmbed;
use crate::utls::parser::ParserResult;

//Traits for compiler lookup
pub trait LanguageResolvable {
    fn resolve(&self, language: &str) -> bool;
}

impl LanguageResolvable for wandbox::Wandbox {
    fn resolve(&self, language: &str) -> bool {
        self.is_valid_language(language) || self.is_valid_compiler_str(language)
    }
}
impl LanguageResolvable for godbolt::Godbolt {
    fn resolve(&self, language: &str) -> bool {
        self.resolve(language).is_some()
    }
}

pub enum RequestHandler {
    None,
    WandBox,
    CompilerExplorer,
}

/// An abstraction for wandbox and godbolt. This object serves as the main interface between
/// any api interactions and will control what languages use what service. Generally how this
/// works is: if the language supported is owned by Compiler Explorer-, we will use them. Otherwise,
/// we fallback on to WandBox to see if they can fulfill the request
pub struct CompilationManager {
    pub wbox: Wandbox,
    pub gbolt: Godbolt,
}

impl CompilationManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let mut broken_compilers = std::collections::HashSet::new();
        broken_compilers.insert(String::from("ghc-head"));
        broken_compilers.insert(String::from("go-head"));
        let mut broken_languages = std::collections::HashSet::new();
        broken_languages.insert(String::from("cpp"));

        Ok(CompilationManager {
            wbox: wandbox::Wandbox::new(Some(broken_compilers), Some(broken_languages)).await?,
            gbolt: Godbolt::new().await?,
        })
    }

    pub async fn compile(
        &self,
        parser_result: &ParserResult,
        author: &User,
    ) -> Result<(String, CreateEmbed), CommandError> {
        return match self.resolve_target(&parser_result.target) {
            RequestHandler::CompilerExplorer => {
                let result = self.compiler_explorer(parser_result).await?;
                Ok((result.0, result.1.to_embed(author, false)))
            }
            RequestHandler::WandBox => {
                let result = self.wandbox(parser_result).await?;
                Ok((result.0, result.1.to_embed(author, false)))
            }
            RequestHandler::None => Err(CommandError::from(format!(
                "Unable to find compiler or language for target '{}'.",
                &parser_result.target
            ))),
        };
    }

    pub async fn assembly(
        &self,
        parse_result: &ParserResult,
        author: &User,
    ) -> Result<(String, CreateEmbed), CommandError> {
        let filters = CompilationFilters {
            binary: None,
            comment_only: Some(true),
            demangle: Some(true),
            directives: Some(true),
            execute: Some(false),
            intel: Some(true),
            labels: Some(true),
            library_code: None,
            trim: Some(true),
        };

        let options = RequestOptions {
            user_arguments: parse_result.options.join(" "),
            compiler_options: CompilerOptions {
                skip_asm: false,
                executor_request: false,
            },
            execute_parameters: Default::default(),
            filters,
        };

        let target = if parse_result.target == "haskell" {
            "ghc901"
        } else {
            &parse_result.target
        };
        let resolution_result = self.gbolt.resolve(target);
        return match resolution_result {
            None => {
                Err(CommandError::from(format!("Target '{}' either does not produce assembly or is not currently supported on godbolt.org", target)))
            }
            Some(compiler) => {
                let response = Godbolt::send_request(&compiler, &parse_result.code, options, USER_AGENT).await?;
                Ok((compiler.lang, response.to_embed(author, true)))
            }
        };
    }

    pub async fn compiler_explorer(
        &self,
        parse_result: &ParserResult,
    ) -> Result<(String, godbolt::GodboltResponse), GodboltError> {
        let filters = CompilationFilters {
            binary: None,
            comment_only: Some(true),
            demangle: Some(true),
            directives: Some(true),
            execute: Some(true),
            intel: Some(true),
            labels: Some(true),
            library_code: None,
            trim: Some(true),
        };

        let options = RequestOptions {
            user_arguments: parse_result.options.join(" "),
            compiler_options: CompilerOptions {
                skip_asm: true,
                executor_request: true,
            },
            execute_parameters: godbolt::ExecuteParameters {
                args: parse_result.args.clone(),
                stdin: parse_result.stdin.clone(),
            },
            filters,
        };

        let target = if parse_result.target == "haskell" {
            "ghc901"
        } else {
            &parse_result.target
        };
        let compiler = self.gbolt.resolve(target).unwrap();

        // replace boilerplate code if needed
        let mut code = parse_result.code.clone();
        {
            let generator = boilerplate_factory(&compiler.lang, &code);
            if generator.needs_boilerplate() {
                code = generator.generate();
            }
        }
        let response = Godbolt::send_request(&compiler, &code, options, USER_AGENT).await?;
        Ok((compiler.lang, response))
    }

    pub fn resolve_target(&self, target: &str) -> RequestHandler {
        if target == "scala" || target == "nim" {
            return RequestHandler::WandBox;
        }

        if self.gbolt.resolve(target).is_some() {
            RequestHandler::CompilerExplorer
        } else if self.wbox.resolve(target) {
            RequestHandler::WandBox
        } else {
            RequestHandler::None
        }
    }

    pub async fn wandbox(
        &self,
        parse_result: &ParserResult,
    ) -> Result<(String, wandbox::CompilationResult), WandboxError> {
        let lang = {
            let mut found = String::default();
            for lang in self.wbox.get_languages() {
                if parse_result.target == lang.name {
                    found = parse_result.target.clone();
                }
                for compiler in lang.compilers {
                    if compiler.name == parse_result.target {
                        found = lang.name.clone();
                    }
                }
            }
            if found.is_empty() {
                warn!("Invalid target leaked checks and was caught before boilerplate creation")
            }
            found
        };
        let mut code = parse_result.code.clone();
        {
            let generator = boilerplate_factory(&lang, &code);
            if generator.needs_boilerplate() {
                code = generator.generate();
            }
        }

        let mut builder = CompilationBuilder::new();
        builder.code(&code);
        builder.target(&parse_result.target);
        builder.stdin(&parse_result.stdin);
        builder.save(false);
        builder.options(parse_result.options.clone());

        builder.build(&self.wbox)?;
        let res = builder.dispatch().await?;
        Ok((builder.lang, res))
    }

    pub fn slash_cmd_langs() -> [&'static str; 11] {
        [
            "Python",
            "C++",
            "Javascript",
            "C",
            "Java",
            "Bash",
            "Lua",
            "C#",
            "Rust",
            "Php",
            "Perl",
        ]
    }
    pub fn slash_cmd_langs_asm() -> [&'static str; 7] {
        ["C++", "C", "Haskell", "Java", "Python", "Rust", "Zig"]
    }
}
