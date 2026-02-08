#![allow(dead_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use clap_complete::{ArgValueCandidates, CompletionCandidate};

use pacs_core::{Pacs, PacsCommand};

const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";
const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// A command-line tool for managing and running saved shell commands.
#[derive(Parser, Debug)]
#[command(name = "pacs")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Launch the terminal user interface
    #[arg(long)]
    pub ui: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize pacs
    Init,

    /// Add a new command
    Add(AddArgs),

    /// Remove a command
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),

    /// Edit an existing command
    Edit(EditArgs),

    /// Rename a command
    Rename(RenameArgs),

    /// List commands
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Run a saved command
    Run(RunArgs),

    /// Copy command to clipboard
    #[command(visible_alias = "cp")]
    Copy(CopyArgs),

    /// Search commands by name or content
    Search(SearchArgs),

    /// Manage projects
    #[command(visible_alias = "p")]
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Manage project-specific environments
    #[command(visible_alias = "e")]
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// Create a new project
    Add(ProjectAddArgs),

    /// Remove a project
    #[command(visible_alias = "rm")]
    Remove(ProjectRemoveArgs),

    /// List all projects
    #[command(visible_alias = "ls")]
    List,

    /// Switch to a project
    Switch(ProjectSwitchArgs),

    /// Clear the active project
    Clear,

    /// Show the current active project
    Active,
}

#[derive(Subcommand, Debug)]
pub enum EnvCommands {
    /// Add a new empty environment to a project
    Add(EnvAddArgs),

    /// Remove an environment from a project
    #[command(visible_alias = "rm")]
    Remove(EnvRemoveArgs),

    /// Edit an environment's values (opens editor)
    Edit(EnvEditArgs),

    /// List environments for a project
    #[command(visible_alias = "ls")]
    List(EnvListArgs),

    /// Switch to an environment
    Switch(EnvSwitchArgs),

    /// Show the active environment for a project
    Active(EnvActiveArgs),
}

#[derive(Args, Debug)]
pub struct ProjectAddArgs {
    /// Name of the project
    pub name: String,

    /// Path associated with the project
    #[arg(short, long)]
    pub path: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProjectRemoveArgs {
    /// Name of the project to remove
    #[arg(add = ArgValueCandidates::new(complete_projects))]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ProjectSwitchArgs {
    /// Name of the project to switch to
    #[arg(add = ArgValueCandidates::new(complete_projects))]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct EnvAddArgs {
    /// Environment name to add (e.g., dev, stg)
    pub name: String,

    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvRemoveArgs {
    /// Environment name to remove
    pub name: String,

    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvEditArgs {
    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvListArgs {
    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvSwitchArgs {
    /// Environment name to switch to
    pub name: String,

    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvActiveArgs {
    /// Target project (defaults to active project if omitted)
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Name for the command
    pub name: String,

    /// The shell command to save
    pub command: Option<String>,

    /// Add to a specific project
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,

    /// Working directory for the command
    #[arg(short, long)]
    pub cwd: Option<String>,

    /// Tag for organizing commands
    #[arg(short, long, default_value = "", add = ArgValueCandidates::new(complete_tags))]
    pub tag: String,
}

#[derive(Args, Debug)]
pub struct CopyArgs {
    /// Name of the command to copy
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub name: String,

    /// Use a specific environment when expanding placeholders
    #[arg(short = 'e', long = "env", add = ArgValueCandidates::new(complete_environments))]
    pub environment: Option<String>,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query (fuzzy matched against name and command)
    pub query: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Name of the command to remove
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Name of the command to edit
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct RenameArgs {
    /// Current name of the command
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub old_name: String,

    /// New name for the command
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Command name to show details for
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub name: Option<String>,

    /// List commands from a specific project only
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,

    /// Filter commands by tag
    #[arg(short, long, add = ArgValueCandidates::new(complete_tags))]
    pub tag: Option<String>,

    /// Show commands resolved for a specific environment (project scope)
    #[arg(short = 'e', long = "env", add = ArgValueCandidates::new(complete_environments))]
    pub environment: Option<String>,

    /// Show only command names (no bodies)
    #[arg(short, long)]
    pub names: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Name of the command to run
    #[arg(add = ArgValueCandidates::new(complete_commands))]
    pub name: String,

    /// Run from a specific project instead of global
    #[arg(short, long, add = ArgValueCandidates::new(complete_projects))]
    pub project: Option<String>,

    /// Use a specific environment for this run
    #[arg(short = 'e', long = "env", add = ArgValueCandidates::new(complete_environments))]
    pub environment: Option<String>,
}

fn complete_commands() -> Vec<CompletionCandidate> {
    let Ok(pacs) = Pacs::init_home() else {
        return vec![];
    };
    pacs.suggest_command_names()
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

fn complete_projects() -> Vec<CompletionCandidate> {
    let Ok(pacs) = Pacs::init_home() else {
        return vec![];
    };
    pacs.suggest_projects()
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

fn complete_tags() -> Vec<CompletionCandidate> {
    let Ok(pacs) = Pacs::init_home() else {
        return vec![];
    };
    pacs.suggest_tags(None)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

fn complete_environments() -> Vec<CompletionCandidate> {
    let Ok(pacs) = Pacs::init_home() else {
        return vec![];
    };
    pacs.suggest_environments(None)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

pub fn run(cli: Cli) -> Result<()> {
    if cli.ui {
        return Ok(());
    }

    let Some(command) = cli.command else {
        use clap::CommandFactory;
        Cli::command().print_help()?;
        println!();
        return Ok(());
    };

    let mut pacs = Pacs::init_home().context("Failed to initialize pacs")?;

    match command {
        Commands::Init => {
            println!("Pacs initialized at ~/.pacs/");

            print!("Enter a name for your first project: ");
            io::stdout().flush()?;
            let mut project_name = String::new();
            io::stdin().read_line(&mut project_name)?;
            let project_name = project_name.trim();

            if project_name.is_empty() {
                anyhow::bail!("No project name entered");
            }

            pacs.init_project(project_name, None)?;
            pacs.set_active_project(project_name)?;
        }

        Commands::Add(args) => {
            let command = if let Some(cmd) = args.command {
                cmd
            } else {
                let editor = env::var("VISUAL")
                    .ok()
                    .or_else(|| env::var("EDITOR").ok())
                    .unwrap_or_else(|| "vi".to_string());

                let temp_file =
                    std::env::temp_dir().join(format!("pacs-{}.sh", std::process::id()));

                fs::write(&temp_file, "")?;

                let status = Command::new(&editor)
                    .arg(&temp_file)
                    .status()
                    .with_context(|| format!("Failed to open editor '{editor}'"))?;

                if !status.success() {
                    fs::remove_file(&temp_file).ok();
                    anyhow::bail!("Editor exited with non-zero status");
                }

                let content = fs::read_to_string(&temp_file)?;
                fs::remove_file(&temp_file).ok();

                let command = content.trim().to_string();

                if command.is_empty() {
                    anyhow::bail!("No command entered");
                }

                command + "\n"
            };

            let pacs_cmd = PacsCommand {
                name: args.name.clone(),
                command,
                cwd: args.cwd,
                tag: args.tag,
            };

            pacs.add_command(pacs_cmd, args.project.as_deref())
                .with_context(|| format!("Failed to add command '{}'", args.name))?;

            let project_name = if let Some(ref p) = args.project {
                p.clone()
            } else {
                pacs.get_active_project_name()?
            };

            println!(
                "Command '{}' added to project '{}'.",
                args.name, project_name
            );
        }

        Commands::Remove(args) => {
            pacs.delete_command_auto(&args.name)
                .with_context(|| format!("Failed to remove command '{}'", args.name))?;
            println!("Command '{}' removed.", args.name);
        }

        Commands::Edit(args) => {
            let cmd = pacs
                .get_command_auto(&args.name)
                .with_context(|| format!("Command '{}' not found", args.name))?;

            let editor = env::var("VISUAL")
                .ok()
                .or_else(|| env::var("EDITOR").ok())
                .unwrap_or_else(|| "vi".to_string());

            let temp_file =
                std::env::temp_dir().join(format!("pacs-edit-{}.sh", std::process::id()));

            fs::write(&temp_file, &cmd.command)?;

            let status = Command::new(&editor)
                .arg(&temp_file)
                .status()
                .with_context(|| format!("Failed to open editor '{editor}'"))?;

            if !status.success() {
                fs::remove_file(&temp_file).ok();
                anyhow::bail!("Editor exited with non-zero status");
            }

            let new_command = fs::read_to_string(&temp_file)?;
            fs::remove_file(&temp_file).ok();

            if new_command.trim().is_empty() {
                anyhow::bail!("Command cannot be empty");
            }

            pacs.update_command_auto(&args.name, new_command)
                .with_context(|| format!("Failed to update command '{}'", args.name))?;
            println!("Command '{}' updated.", args.name);
        }

        Commands::Rename(args) => {
            pacs.rename_command_auto(&args.old_name, &args.new_name)
                .with_context(|| {
                    format!(
                        "Failed to rename command '{}' to '{}'",
                        args.old_name, args.new_name
                    )
                })?;
            println!(
                "Command '{}' renamed to '{}'.",
                args.old_name, args.new_name
            );
        }

        Commands::List(args) => {
            if let Some(ref name) = args.name {
                let cmd = pacs
                    .resolve_command(name, None, args.environment.as_deref())
                    .with_context(|| format!("Command '{name}' not found"))?;
                let tag_badge = if cmd.tag.is_empty() {
                    String::new()
                } else {
                    format!(" {BOLD}{YELLOW}[{}]{RESET}", cmd.tag)
                };
                let cwd_badge = if let Some(ref cwd) = cmd.cwd {
                    format!(" {GREY}({cwd}){RESET}")
                } else {
                    String::new()
                };
                println!("{BOLD}{CYAN}{}{RESET}{}{}", cmd.name, tag_badge, cwd_badge);
                for line in cmd.command.lines() {
                    println!("{WHITE}{line}{RESET}");
                }
                return Ok(());
            }

            let filter_tag =
                |cmd: &PacsCommand| -> bool { args.tag.as_ref().is_none_or(|t| &cmd.tag == t) };

            let print_tagged = |commands: &[PacsCommand], scope_name: &str| {
                if commands.is_empty() {
                    println!("No commands found. Use 'pacs add <name> <cmd>' to add one.");
                    return;
                }

                let mut tags: BTreeMap<Option<&str>, Vec<&PacsCommand>> = BTreeMap::new();
                for cmd in commands.iter().filter(|c| filter_tag(c)) {
                    let key = if cmd.tag.is_empty() {
                        None
                    } else {
                        Some(cmd.tag.as_str())
                    };
                    tags.entry(key).or_default().push(cmd);
                }

                if tags.is_empty() {
                    return;
                }

                println!("{BOLD}{GREEN}{scope_name}{RESET}{RESET}");
                println!();

                for (tag, cmds) in tags {
                    if let Some(name) = tag {
                        println!("{BOLD}{YELLOW}[{name}]{RESET}");
                    }

                    for cmd in cmds {
                        if args.names {
                            println!("{BOLD}{CYAN}{}{RESET}", cmd.name);
                        } else {
                            let cwd_badge = if let Some(ref cwd) = cmd.cwd {
                                format!(" {GREY}({cwd}){RESET}")
                            } else {
                                String::new()
                            };
                            println!("{BOLD}{CYAN}{}{RESET}{}", cmd.name, cwd_badge);
                            for line in cmd.command.lines() {
                                println!("{WHITE}{line}{RESET}");
                            }
                            println!();
                        }
                    }
                }
            };

            if let Some(ref project) = args.project {
                let commands = pacs.list(Some(project), args.environment.as_deref())?;
                print_tagged(&commands, project);
            } else {
                let active_project =   pacs.get_active_project_name().context("No active project. Use 'pacs project add' to create one or 'pacs project switch' to activate one.")?;
                let commands = pacs.list(None, args.environment.as_deref())?;
                print_tagged(&commands, &active_project);
            }
        }

        Commands::Run(args) => {
            pacs.run(
                &args.name,
                args.project.as_deref(),
                args.environment.as_deref(),
            )
            .with_context(|| format!("Failed to run command '{}'", args.name))?;
        }

        Commands::Copy(args) => {
            let cmd = pacs
                .copy(&args.name, None, args.environment.as_deref())
                .with_context(|| format!("Command '{}' not found", args.name))?;
            arboard::Clipboard::new()
                .and_then(|mut cb| cb.set_text(cmd.command.trim()))
                .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {e}"))?;
            println!("Copied '{}' to clipboard.", args.name);
        }

        Commands::Search(args) => {
            let matches = pacs.search(&args.query);
            if matches.is_empty() {
                println!("No matches found.");
            } else {
                for cmd in matches {
                    println!("{}", cmd.name);
                }
            }
        }

        Commands::Project { command } => match command {
            ProjectCommands::Add(args) => {
                pacs.init_project(&args.name, args.path)
                    .with_context(|| format!("Failed to create project '{}'", args.name))?;
                pacs.set_active_project(&args.name)
                    .with_context(|| format!("Failed to switch to project '{}'", args.name))?;
                println!("Project '{}' created and activated.", args.name);
            }
            ProjectCommands::Remove(args) => {
                pacs.delete_project(&args.name)
                    .with_context(|| format!("Failed to delete project '{}'", args.name))?;
                println!("Project '{}' deleted.", args.name);
            }
            ProjectCommands::List => {
                if pacs.projects.is_empty() {
                    println!("No projects. Use 'pacs project add' to create one.");
                } else {
                    let active = pacs.get_active_project_name().ok();
                    for project in &pacs.projects {
                        let path_info = project
                            .path
                            .as_ref()
                            .map(|p| format!(" ({p})"))
                            .unwrap_or_default();
                        let active_marker = if active.as_ref() == Some(&project.name) {
                            format!(" {GREEN}*{RESET}")
                        } else {
                            String::new()
                        };
                        println!(
                            "{}{}{}{}{}",
                            BLUE, project.name, RESET, path_info, active_marker
                        );
                    }
                }
            }
            ProjectCommands::Switch(args) => {
                pacs.set_active_project(&args.name)
                    .with_context(|| format!("Failed to switch to project '{}'", args.name))?;
                println!("Switched to project '{}'.", args.name);
            }
            ProjectCommands::Clear => {
                pacs.clear_active_project()?;
                println!("Active project cleared.");
            }
            ProjectCommands::Active => match pacs.get_active_project_name() {
                Ok(active) => println!("{active}"),
                Err(_) => println!("No active project."),
            },
        },
        Commands::Env { command } => match command {
            EnvCommands::Add(args) => {
                let project = resolve_project_name(&pacs, args.project)?;

                pacs.add_environment(&project, &args.name)
                    .with_context(|| {
                        format!(
                            "Failed to add environment '{}' to project '{}'",
                            args.name, project
                        )
                    })?;
                pacs.set_active_environment(&project, &args.name)
                    .with_context(|| {
                        format!(
                            "Failed to activate environment '{}' in project '{}'",
                            args.name, project
                        )
                    })?;
                println!(
                    "Environment '{}' added and activated in project '{}'.",
                    args.name, project
                );
            }
            EnvCommands::Remove(args) => {
                let project = resolve_project_name(&pacs, args.project)?;

                pacs.remove_environment(&project, &args.name)
                    .with_context(|| {
                        format!(
                            "Failed to remove environment '{}' from project '{}'",
                            args.name, project
                        )
                    })?;
                println!(
                    "Environment '{}' removed from project '{}'.",
                    args.name, project
                );
            }
            EnvCommands::Edit(args) => {
                #[derive(serde::Deserialize)]
                struct EditDoc {
                    #[serde(default)]
                    active_environment: Option<String>,
                    #[serde(default)]
                    environments: std::collections::BTreeMap<String, EnvValues>,
                }
                #[derive(serde::Deserialize)]
                struct EnvValues {
                    #[serde(default)]
                    values: BTreeMap<String, String>,
                }

                let editor = env::var("VISUAL")
                    .ok()
                    .or_else(|| env::var("EDITOR").ok())
                    .unwrap_or_else(|| "vi".to_string());

                let project = resolve_project_name(&pacs, args.project)?;

                let project_ref = pacs
                    .projects
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(&project))
                    .with_context(|| format!("Project '{project}' not found"))?;

                let mut buf = String::new();
                if let Some(active_env) = &project_ref.active_environment {
                    write!(buf, "active_environment = \"{active_env}\"\n\n").unwrap();
                }

                for env in &project_ref.environments {
                    writeln!(buf, "[environments.{}.values]", env.name).unwrap();
                    for (k, v) in &env.values {
                        writeln!(buf, "{k} = \"{}\"", v.replace('"', "\\\"")).unwrap();
                    }
                    buf.push('\n');
                }

                let temp_file =
                    std::env::temp_dir().join(format!("pacs-env-{}.toml", std::process::id()));
                fs::write(&temp_file, buf)?;

                let status = Command::new(&editor)
                    .arg(&temp_file)
                    .status()
                    .with_context(|| format!("Failed to open editor '{editor}'"))?;

                if !status.success() {
                    fs::remove_file(&temp_file).ok();
                    anyhow::bail!("Editor exited with non-zero status");
                }

                let edited = fs::read_to_string(&temp_file)?;
                fs::remove_file(&temp_file).ok();

                let doc: EditDoc =
                    toml::from_str(&edited).with_context(|| "Failed to parse edited TOML")?;

                if let Some(active_name) = doc.active_environment {
                    pacs.set_active_environment(&project, &active_name)
                        .with_context(|| {
                            format!("Failed to set active environment '{active_name}'")
                        })?;
                }

                for (env_name, env_values) in doc.environments {
                    pacs.edit_environment_values(&project, &env_name, env_values.values.clone())
                        .with_context(|| {
                            format!(
                                "Failed to update environment '{env_name}' values for project '{project}'"
                            )
                        })?;
                }
                println!("All environments updated for project '{project}'.");
            }
            EnvCommands::List(args) => {
                let environments = pacs
                    .list_environments(args.project.as_deref())
                    .context("Failed to list environments")?;
                let active = pacs
                    .get_active_environment(args.project.as_deref())
                    .context("Failed to get active environment")?;

                if environments.is_empty() {
                    println!("No environments.");
                } else {
                    for env in environments {
                        let active_marker = if active.as_deref() == Some(env.name.as_str()) {
                            format!(" {GREEN}*{RESET}")
                        } else {
                            String::new()
                        };
                        println!("{CYAN}{BOLD}{}{active_marker}{RESET}", env.name);
                        if !env.values.is_empty() {
                            for (k, v) in &env.values {
                                println!("  {GREY}{k}{RESET} = {WHITE}{v}{RESET}");
                            }
                        }
                    }
                }
            }
            EnvCommands::Switch(args) => {
                let project = resolve_project_name(&pacs, args.project)?;

                pacs.set_active_environment(&project, &args.name)
                    .with_context(|| {
                        format!(
                            "Failed to switch to environment '{}' in project '{}'",
                            args.name, project
                        )
                    })?;
                println!(
                    "Switched to environment '{}' in project '{}'.",
                    args.name, project
                );
            }
            EnvCommands::Active(args) => {
                let project = resolve_project_name(&pacs, args.project)?;

                match pacs.get_active_environment(Some(&project))? {
                    Some(name) => println!("{name}"),
                    None => println!("No active environment."),
                }
            }
        },
    }

    Ok(())
}

fn resolve_project_name(pacs: &Pacs, project_name: Option<String>) -> Result<String> {
    match project_name {
        Some(p) => Ok(p),
        None => pacs.get_active_project_name().map_err(|_| {
            anyhow::anyhow!(
                "No project specified and no active project set. \
                    Use 'pacs project add' to create one or 'pacs project switch' to activate one."
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
