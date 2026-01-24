#![allow(dead_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use clap_complete::{ArgValueCandidates, CompletionCandidate};

use pacs_core::{Pacs, PacsCommand, Scope};

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
    #[command(subcommand)]
    pub command: Commands,
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
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
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

    /// Set a project as active
    Activate(ProjectActivateArgs),

    /// Clear the active project
    Deactivate,

    /// Show the current active project
    Active,
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
pub struct ProjectActivateArgs {
    /// Name of the project to activate
    #[arg(add = ArgValueCandidates::new(complete_projects))]
    pub name: String,
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

    /// Add to global scope (default: adds to active project if set, otherwise global)
    #[arg(short, long)]
    pub global: bool,

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

    /// List only global commands
    #[arg(short, long)]
    pub global: bool,

    /// Filter commands by tag
    #[arg(short, long, add = ArgValueCandidates::new(complete_tags))]
    pub tag: Option<String>,

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
    pacs.suggest_tags()
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

pub fn run(cli: Cli) -> Result<()> {
    let mut pacs = Pacs::init_home().context("Failed to initialize pacs")?;

    match cli.command {
        Commands::Init => {
            println!("Pacs initialized at ~/.pacs/");
        }

        Commands::Add(args) => {
            let command = if let Some(cmd) = args.command {
                cmd
            } else {
                let editor = env::var("VISUAL")
                    .or_else(|_| env::var("EDITOR"))
                    .unwrap_or_else(|_| "vi".to_string());

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

            // Determine scope: explicit project > global flag > active project > global
            let scope_name: Option<String> = if let Some(ref p) = args.project {
                Some(p.clone())
            } else if args.global {
                None
            } else {
                pacs.get_active_project()?
            };

            if let Some(ref project) = scope_name {
                pacs.add_command(pacs_cmd, Scope::Project(project))
                    .with_context(|| format!("Failed to add command '{}'", args.name))?;
                println!("Command '{}' added to project '{}'.", args.name, project);
            } else {
                pacs.add_command(pacs_cmd, Scope::Global)
                    .with_context(|| format!("Failed to add command '{}'", args.name))?;
                println!("Command '{}' added to global.", args.name);
            }
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
                .or_else(|_| env::var("EDITOR"))
                .unwrap_or_else(|_| "vi".to_string());

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
                    .get_command_auto(name)
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
                println!();
                for line in cmd.command.lines() {
                    println!("{BLUE}{line}{RESET}");
                }
                return Ok(());
            }

            let filter_tag =
                |cmd: &PacsCommand| -> bool { args.tag.as_ref().is_none_or(|t| &cmd.tag == t) };

            let print_tagged = |commands: &[&PacsCommand], scope_name: &str| {
                if commands.is_empty() {
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

                println!("{BOLD}{MAGENTA}── {scope_name} ──{RESET}");

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
                                println!("{BLUE}{line}{RESET}");
                            }
                            println!();
                        }
                    }
                }
            };

            if let Some(ref project) = args.project {
                let commands = pacs.list_commands(Scope::Project(project))?;
                print_tagged(&commands, project);
            } else if args.global {
                let commands = pacs.list_commands(Scope::Global)?;
                print_tagged(&commands, "Global");
            } else {
                let commands = pacs.list_commands(Scope::Global)?;
                print_tagged(&commands, "Global");

                if let Some(active_project) = pacs.get_active_project()? {
                    let commands = pacs.list_commands(Scope::Project(&active_project))?;
                    print_tagged(&commands, &active_project);
                } else {
                    for project in &pacs.projects {
                        let commands = pacs.list_commands(Scope::Project(&project.name))?;
                        print_tagged(&commands, &project.name);
                    }
                }
            }
        }

        Commands::Run(args) => {
            if let Some(ref project) = args.project {
                pacs.run(&args.name, Scope::Project(project))
                    .with_context(|| format!("Failed to run command '{}'", args.name))?;
            } else {
                pacs.run_auto(&args.name)
                    .with_context(|| format!("Failed to run command '{}'", args.name))?;
            }
        }

        Commands::Copy(args) => {
            let cmd = pacs
                .get_command_auto(&args.name)
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
                println!("Project '{}' created.", args.name);
            }
            ProjectCommands::Remove(args) => {
                pacs.delete_project(&args.name)
                    .with_context(|| format!("Failed to delete project '{}'", args.name))?;
                println!("Project '{}' deleted.", args.name);
            }
            ProjectCommands::List => {
                if pacs.projects.is_empty() {
                    println!("No projects.");
                } else {
                    let active = pacs.get_active_project().ok().flatten();
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
            ProjectCommands::Activate(args) => {
                pacs.set_active_project(&args.name)
                    .with_context(|| format!("Failed to activate project '{}'", args.name))?;
                println!("Project '{}' is now active.", args.name);
            }
            ProjectCommands::Deactivate => {
                pacs.clear_active_project()
                    .context("Failed to deactivate project")?;
                println!("Active project cleared.");
            }
            ProjectCommands::Active => {
                if let Some(active) = pacs.get_active_project()? {
                    println!("{active}");
                } else {
                    println!("No active project.");
                }
            }
        },
    }

    Ok(())
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
