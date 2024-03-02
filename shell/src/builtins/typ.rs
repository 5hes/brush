use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use parser::ast;

use crate::{
    builtin::{BuiltinCommand, BuiltinExitCode},
    Shell,
};

#[derive(Parser, Debug)]
pub(crate) struct TypeCommand {
    #[arg(short = 'a')]
    all_locations: bool,

    #[arg(short = 'f')]
    suppress_func_lookup: bool,

    #[arg(short = 'P')]
    force_path_search: bool,

    #[arg(short = 'p')]
    show_path_only: bool,

    #[arg(short = 't')]
    type_only: bool,

    names: Vec<String>,
}

enum ResolvedType {
    Alias(String),
    Keyword,
    Function(ast::FunctionDefinition),
    Builtin,
    File(PathBuf),
}

#[async_trait::async_trait]
impl BuiltinCommand for TypeCommand {
    async fn execute(
        &self,
        context: &mut crate::builtin::BuiltinExecutionContext<'_>,
    ) -> Result<crate::builtin::BuiltinExitCode, crate::error::Error> {
        let mut result = BuiltinExitCode::Success;

        for name in &self.names {
            let resolved_types = self.resolve_types(context.shell, name);

            if resolved_types.is_empty() {
                if !self.type_only && !self.force_path_search {
                    log::error!("type: {name} not found");
                }

                result = BuiltinExitCode::Custom(1);
                continue;
            }

            for resolved_type in resolved_types {
                if self.show_path_only && !matches!(resolved_type, ResolvedType::File(_)) {
                    // Do nothing.
                } else if self.type_only {
                    match resolved_type {
                        ResolvedType::Alias(_) => println!("alias"),
                        ResolvedType::Keyword => println!("keyword"),
                        ResolvedType::Function(_) => println!("function"),
                        ResolvedType::Builtin => println!("builtin"),
                        ResolvedType::File(path) => {
                            if self.show_path_only || self.force_path_search {
                                println!("{}", path.to_string_lossy());
                            } else {
                                println!("file");
                            }
                        }
                    }
                } else {
                    match resolved_type {
                        ResolvedType::Alias(target) => println!("{name} is aliased to '{target}'"),
                        ResolvedType::Keyword => println!("{name} is a shell keyword"),
                        ResolvedType::Function(def) => {
                            println!("{name} is a function");
                            println!("{def}");
                        }
                        ResolvedType::Builtin => println!("{name} is a shell builtin"),
                        ResolvedType::File(path) => {
                            if self.show_path_only || self.force_path_search {
                                println!("{}", path.to_string_lossy());
                            } else {
                                println!(
                                    "{name} is {path}",
                                    name = name,
                                    path = path.to_string_lossy()
                                );
                            }
                        }
                    }
                }

                // If we only want the first, then break after the first.
                if !self.all_locations {
                    break;
                }
            }
        }

        Ok(result)
    }
}

impl TypeCommand {
    fn resolve_types(&self, shell: &Shell, name: &str) -> Vec<ResolvedType> {
        let mut types = vec![];

        if !self.force_path_search {
            // Check for aliases.
            if let Some(a) = shell.aliases.get(name) {
                types.push(ResolvedType::Alias(a.clone()));
            }

            // Check for keywords.
            if is_keyword(name) {
                types.push(ResolvedType::Keyword);
            }

            // Check for functions.
            if !self.suppress_func_lookup {
                if let Some(def) = shell.funcs.get(name) {
                    types.push(ResolvedType::Function(def.clone()));
                }
            }

            // Check for builtins.
            if crate::builtins::SPECIAL_BUILTINS.contains_key(name)
                || crate::builtins::BUILTINS.contains_key(name)
            {
                types.push(ResolvedType::Builtin);
            }
        }

        // Look in path.
        if name.contains('/') {
            // TODO: Handle this case.
        } else {
            for item in shell.find_executables_in_path(name) {
                types.push(ResolvedType::File(item));
            }
        }

        types
    }
}

fn is_keyword(name: &str) -> bool {
    match name {
        "!" => true,
        "{" => true,
        "}" => true,
        "case" => true,
        "do" => true,
        "done" => true,
        "elif" => true,
        "else" => true,
        "esac" => true,
        "fi" => true,
        "for" => true,
        "if" => true,
        "in" => true,
        "then" => true,
        "until" => true,
        "while" => true,
        // N.B. bash also treats the following as reserved.
        // TODO: Disable these in POSIX compliance mode.
        "[[" => true,
        "]]" => true,
        "function" => true,
        "select" => true,
        _ => false,
    }
}