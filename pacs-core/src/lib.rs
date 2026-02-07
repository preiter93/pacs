//! # PACS Core
//!
//! ## API
//!
//! **Command Management:**
//! - `add_command(cmd, project_name)` - Add a command to a project
//! - `delete_command(name, project_name)` - Remove a command from a project
//! - `list(project_name, environment)` - List all commands in a project
//! - `run(name, project_name, environment)` - Execute a command
//! - `copy(name, project_name, environment)` - Get command text for clipboard
//!
//! **Project Management:**
//! - `init_project(name, path)` - Create a new project
//! - `delete_project(name)` - Remove a project and all its commands
//! - `set_active_project(name)` - Set the active project
//! - `get_active_project()` - Get the current active project name
//!
//! **Environment Management:**
//! - `add_environment(project_name, env_name)` - Add an environment to a project
//! - `remove_environment(project_name, env_name)` - Remove an environment
//! - `activate_environment(project_name, env_name)` - Set active environment for a project
//! - `edit_environment_values(project_name, env_name, values)` - Update environment values
//!
//! ### Auto Functions (use active project)
//!
//! These helper functions operate on the active project and don't accept a project parameter:
//! - `get_command_auto(name)` - Find a command in the active project
//! - `update_command_auto(name, command)` - Update a command in the active project
//! - `rename_command_auto(old, new)` - Rename a command in the active project
//! - `delete_command_auto(name)` - Delete a command from the active project

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct as _};
use std::{fs, path::PathBuf, process::Command};
use thiserror::Error;

/// Type alias for project names
pub type ProjectName<'a> = &'a str;

/// Type alias for environment names (e.g., "dev", "staging", "prod")
pub type EnvironmentName<'a> = &'a str;

#[derive(Error, Debug)]
pub enum PacsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command already exists: {0}")]
    CommandExists(String),

    #[error("Command is marked as dangerous: {0}")]
    DangerousCommand(String),

    #[error("Command execution failed with status: {0}")]
    CommandFailed(i32),

    #[error("Unresolved placeholders: {0}")]
    UnresolvedPlaceholders(String),

    #[error("Could not determine home directory")]
    HomeDirUnavailable,

    #[error("Project already exists: {0}")]
    ProjectExists(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("No active project set")]
    NoActiveProject,
}

/// A saved shell command that can be executed.
#[derive(Debug, Deserialize, Clone)]
pub struct PacsCommand {
    /// Unique identifier for this command within its project.
    pub name: String,
    /// The shell command to execute. Can contain `{{placeholder}}` values.
    pub command: String,
    /// Working directory for execution. Uses current directory if None.
    pub cwd: Option<String>,
    /// Optional tag for organization.
    #[serde(default)]
    pub tag: String,
}

impl Serialize for PacsCommand {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("PacsCommand", 4)?;
        s.serialize_field("name", &self.name)?;

        // Append a newline so toml serializes this string as a multiline block
        let mut command = self.command.clone();
        if !command.ends_with('\n') {
            command.push('\n');
        }

        s.serialize_field("cwd", &self.cwd)?;
        s.serialize_field("tag", &self.tag)?;
        s.serialize_field("command", &command)?;
        s.end()
    }
}

impl PacsCommand {
    /// Finds a command by name in a slice.
    pub fn find_by_name<'a>(
        commands: &'a [PacsCommand],
        name: &str,
    ) -> Result<&'a PacsCommand, PacsError> {
        commands
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| PacsError::CommandNotFound(name.to_string()))
    }

    /// Finds a mutable command by name in a slice.
    pub fn find_by_name_mut<'a>(
        commands: &'a mut [PacsCommand],
        name: &str,
    ) -> Result<&'a mut PacsCommand, PacsError> {
        commands
            .iter_mut()
            .find(|c| c.name == name)
            .ok_or_else(|| PacsError::CommandNotFound(name.to_string()))
    }
}

/// Environment values for a named project environment.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Environment {
    /// Environment identifier (e.g., "dev", "stg").
    pub name: String,
    /// Key-value pairs used to render placeholders like `{key}`.
    #[serde(default)]
    pub values: std::collections::BTreeMap<String, String>,
}

/// A collection of commands associated with a project.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Project {
    /// Unique project identifier.
    pub name: String,
    /// Optional filesystem path associated with this project.
    pub path: Option<String>,
    /// Commands belonging to this project.
    #[serde(default)]
    pub commands: Vec<PacsCommand>,
    /// Environments defined for this project.
    #[serde(default)]
    pub environments: Vec<Environment>,
    /// The active environment name used to render placeholders for this project.
    #[serde(default)]
    pub active_environment: Option<String>,
}

/// Configuration stored in config.toml
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// The currently active project name.
    #[serde(default)]
    pub active_project: Option<String>,
}

/// Main container managing projects and their commands.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Pacs {
    /// Registered projects with their own commands.
    pub projects: Vec<Project>,
    #[serde(skip)]
    base_dir: PathBuf,
}

impl Pacs {
    /// Initializes Pacs home directory at ~/.pacs/
    pub fn init_home() -> Result<Self, PacsError> {
        let mut base = dirs::home_dir().ok_or(PacsError::HomeDirUnavailable)?;
        base.push(".pacs");
        Self::init_at(base)
    }

    /// Initializes Pacs at a custom base path.
    pub fn init_at(base: PathBuf) -> Result<Self, PacsError> {
        let projects_dir = base.join("projects");

        if !base.exists() {
            fs::create_dir_all(&projects_dir)?;
        }

        let projects = Self::load_projects(&projects_dir)?;

        Ok(Self {
            projects,
            base_dir: base,
        })
    }

    /// Loads the config from config.toml.
    fn load_config(&self) -> Result<Config, PacsError> {
        let path = self.base_dir.join("config.toml");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            if content.trim().is_empty() {
                Ok(Config::default())
            } else {
                Ok(toml::from_str(&content)?)
            }
        } else {
            Ok(Config::default())
        }
    }

    /// Saves the config to config.toml.
    fn save_config(&self, config: &Config) -> Result<(), PacsError> {
        fs::write(
            self.base_dir.join("config.toml"),
            toml::to_string_pretty(config)?,
        )?;
        Ok(())
    }

    /// Sets the active project by name.
    pub fn set_active_project(&self, name: ProjectName) -> Result<(), PacsError> {
        self.get_project(name)?;
        let mut config = self.load_config()?;
        config.active_project = Some(name.to_string());
        self.save_config(&config)?;
        Ok(())
    }

    /// Clears the active project.
    pub fn clear_active_project(&self) -> Result<(), PacsError> {
        let mut config = self.load_config()?;
        config.active_project = None;
        self.save_config(&config)?;
        Ok(())
    }

    /// Returns the name of the active project.
    pub fn get_active_project_name(&self) -> Result<String, PacsError> {
        let config = self.load_config()?;
        let name = config.active_project.ok_or(PacsError::NoActiveProject)?;
        self.get_project(&name)?;
        Ok(name)
    }

    /// Returns a reference to the active project.
    pub fn get_active_project(&self) -> Result<&Project, PacsError> {
        let config = self.load_config()?;
        let name = config.active_project.ok_or(PacsError::NoActiveProject)?;
        self.get_project(&name)
    }

    /// Returns a mutable reference to the active project.
    pub fn get_active_project_mut(&mut self) -> Result<&mut Project, PacsError> {
        let name = self.get_active_project_name()?;
        self.get_project_mut(&name)
    }

    /// Creates a new project with the given name and optional path.
    pub fn init_project(
        &mut self,
        name: ProjectName,
        path: Option<String>,
    ) -> Result<(), PacsError> {
        if self.projects.iter().any(|p| p.name == name) {
            return Err(PacsError::ProjectExists(name.to_string()));
        }

        let project = Project {
            name: name.to_string(),
            path,
            commands: Vec::new(),
            environments: Vec::new(),
            active_environment: None,
        };

        self.save_project(&project)?;
        self.projects.push(project);
        Ok(())
    }

    /// Removes a project and its associated file.
    pub fn delete_project(&mut self, name: ProjectName) -> Result<(), PacsError> {
        let idx = self
            .projects
            .iter()
            .position(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| PacsError::ProjectNotFound(name.to_string()))?;

        self.projects.remove(idx);

        let path = self.project_path(name);
        if path.exists() {
            fs::remove_file(path)?;
        }

        // Clear active project config if it was the deleted one
        let config = self.load_config()?;
        if config
            .active_project
            .is_some_and(|a| a.to_lowercase() == name.to_lowercase())
        {
            self.clear_active_project()?;
        }

        Ok(())
    }

    /// Adds a command to the specified project, or the active project if none specified.
    /// Returns an error if a command with the same name already exists in the project.
    pub fn add_command(
        &mut self,
        cmd: PacsCommand,
        project_name: Option<ProjectName>,
    ) -> Result<(), PacsError> {
        let project_name = match project_name {
            Some(name) => name,
            None => &self.get_active_project_name()?,
        };

        let project = self.get_project_mut(project_name)?;
        if project.commands.iter().any(|c| c.name == cmd.name) {
            return Err(PacsError::CommandExists(cmd.name));
        }

        project.commands.push(cmd);
        self.save_project_by_name(project_name)?;
        Ok(())
    }

    /// Removes a command by name from the specified project, or the active project if none specified.
    pub fn delete_command(
        &mut self,
        command_name: &str,
        project_name: Option<ProjectName>,
    ) -> Result<(), PacsError> {
        let project_name = match project_name {
            Some(name) => name,
            None => &self.get_active_project_name()?,
        };

        let project = self.get_project_mut(project_name)?;

        let before = project.commands.len();
        project.commands.retain(|c| c.name != command_name);

        if project.commands.len() == before {
            return Err(PacsError::CommandNotFound(command_name.to_string()));
        }

        self.save_project_by_name(project_name)?;
        Ok(())
    }

    /// Updates a command's content in the active project.
    pub fn update_command_auto(
        &mut self,
        name: &str,
        new_command: String,
    ) -> Result<String, PacsError> {
        let project = self.get_active_project_mut()?;
        let project_name = project.name.clone();

        let cmd = find_command_mut(project, name)?;

        let old_command = cmd.command.clone();
        cmd.command = new_command;

        self.save_project_by_name(&project_name)?;
        Ok(old_command)
    }

    pub fn rename_command_auto(&mut self, old_name: &str, new_name: &str) -> Result<(), PacsError> {
        let project = self.get_active_project_mut()?;
        let project_name = project.name.clone();

        if project.commands.iter().any(|c| c.name == new_name) {
            return Err(PacsError::CommandExists(new_name.to_string()));
        }

        if !project.commands.iter().any(|c| c.name == old_name) {
            return Err(PacsError::CommandNotFound(old_name.to_string()));
        }

        let cmd = find_command_mut(project, old_name)?;
        cmd.name = new_name.to_string();

        self.save_project_by_name(&project_name)?;
        Ok(())
    }

    pub fn get_command_auto(&self, name: &str) -> Result<&PacsCommand, PacsError> {
        let project = self.get_active_project()?;

        if let Some(cmd) = project.commands.iter().find(|c| c.name == name) {
            return Ok(cmd);
        }

        Err(PacsError::CommandNotFound(name.to_string()))
    }

    pub fn delete_command_auto(&mut self, name: &str) -> Result<(), PacsError> {
        let project = self.get_active_project_mut()?;
        let project_name = project.name.clone();

        let before = project.commands.len();
        project.commands.retain(|c| c.name != name);
        if project.commands.len() == before {
            return Err(PacsError::CommandNotFound(name.to_string()));
        }

        self.save_project_by_name(&project_name)?;
        Ok(())
    }

    pub fn list(
        &self,
        project_name: Option<ProjectName>,
        environment: Option<EnvironmentName>,
    ) -> Result<Vec<PacsCommand>, PacsError> {
        let project_name = match project_name {
            Some(name) => name,
            None => &self.get_active_project_name()?,
        };

        let project = self.get_project(project_name)?;
        let environment = environment.or(project.active_environment.as_deref());

        let mut cmds: Vec<PacsCommand> = Vec::with_capacity(project.commands.len());

        for c in &project.commands {
            let pc = self.expand_command_with_environment(c, project_name, environment)?;
            cmds.push(pc);
        }

        cmds.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(cmds)
    }

    /// Resolves a command with environment, returning an expanded command ready to execute.
    /// Requires an active project if `project_name` is not specified.
    pub fn resolve_command(
        &self,
        name: &str,
        project_name: Option<ProjectName>,
        environment: Option<EnvironmentName>,
    ) -> Result<PacsCommand, PacsError> {
        let project_name = match project_name {
            Some(name) => name,
            None => &self.get_active_project_name()?,
        };

        let project = self.get_project(project_name)?;
        let environment = environment.or(project.active_environment.as_deref());

        let cmd = project
            .commands
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| PacsError::CommandNotFound(name.to_string()))?;

        self.expand_command_with_environment(cmd, project_name, environment)
    }

    pub fn run(
        &self,
        name: &str,
        project_name: Option<ProjectName>,
        environment: Option<EnvironmentName>,
    ) -> Result<(), PacsError> {
        let command = self.resolve_command(name, project_name, environment)?;
        Self::execute(&command)
    }

    pub fn copy(
        &self,
        name: &str,
        project_name: Option<ProjectName>,
        environment: Option<EnvironmentName>,
    ) -> Result<PacsCommand, PacsError> {
        self.resolve_command(name, project_name, environment)
    }

    fn load_projects(projects_dir: &std::path::Path) -> Result<Vec<Project>, PacsError> {
        let mut projects = Vec::new();

        if !projects_dir.exists() {
            return Ok(projects);
        }

        for entry in fs::read_dir(projects_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            let mut proj: Project = toml::from_str(&fs::read_to_string(&path)?)?;
            if proj.name.is_empty()
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                proj.name = stem.to_string();
            }
            projects.push(proj);
        }

        Ok(projects)
    }

    fn get_project_mut(&mut self, name: ProjectName) -> Result<&mut Project, PacsError> {
        self.projects
            .iter_mut()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| PacsError::ProjectNotFound(name.to_string()))
    }

    fn get_project(&self, name: ProjectName) -> Result<&Project, PacsError> {
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| PacsError::ProjectNotFound(name.to_string()))
    }

    fn project_path(&self, name: ProjectName) -> PathBuf {
        self.base_dir.join("projects").join(format!("{name}.toml"))
    }

    fn save_project(&self, project: &Project) -> Result<(), PacsError> {
        let mut sorted = project.commands.clone();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        let temp = Project {
            name: project.name.clone(),
            path: project.path.clone(),
            commands: sorted,
            environments: project.environments.clone(),
            active_environment: project.active_environment.clone(),
        };
        fs::write(
            self.project_path(&project.name),
            toml::to_string_pretty(&temp)?,
        )?;
        Ok(())
    }

    pub fn save_project_by_name(&self, name: ProjectName) -> Result<(), PacsError> {
        let project = self.get_project(name)?;
        self.save_project(project)
    }

    /// Adds a new empty environment to a project.
    pub fn add_environment(
        &mut self,
        project_name: ProjectName,
        environment_name: EnvironmentName,
    ) -> Result<(), PacsError> {
        let project = self.get_project_mut(project_name)?;
        if project
            .environments
            .iter()
            .any(|e| e.name == environment_name)
        {
            return Err(PacsError::ProjectExists(format!(
                "Environment '{environment_name}' already exists in project '{project_name}'"
            )));
        }
        project.environments.push(Environment {
            name: environment_name.to_string(),
            values: std::collections::BTreeMap::new(),
        });

        self.save_project_by_name(project_name)
    }

    /// Removes an existing environment from a project.
    pub fn remove_environment(
        &mut self,
        project_name: ProjectName,
        environment_name: EnvironmentName,
    ) -> Result<(), PacsError> {
        let project = self.get_project_mut(project_name)?;
        if let Some(idx) = project
            .environments
            .iter()
            .position(|e| e.name == environment_name)
        {
            project.environments.remove(idx);
            // If the removed environment was active, deactivate it.
            if project.active_environment.as_deref() == Some(environment_name) {
                project.active_environment = None;
            }
        } else {
            return Err(PacsError::ProjectNotFound(format!(
                "Environment '{environment_name}' not found in project '{project_name}'"
            )));
        }

        self.save_project_by_name(project_name)
    }

    /// Replaces all key/value pairs in a project's environment.
    pub fn edit_environment_values(
        &mut self,
        project_name: ProjectName,
        environment_name: EnvironmentName,
        values: std::collections::BTreeMap<String, String>,
    ) -> Result<(), PacsError> {
        let project = self.get_project_mut(project_name)?;
        let env = project
            .environments
            .iter_mut()
            .find(|e| e.name == environment_name)
            .ok_or_else(|| {
                PacsError::ProjectNotFound(format!(
                    "Environment '{environment_name}' not found in project '{project_name}'"
                ))
            })?;
        env.values = values;

        self.save_project_by_name(project_name)
    }

    /// Activates a specific environment for a project.
    pub fn activate_environment(
        &mut self,
        project_name: ProjectName,
        environment_name: EnvironmentName,
    ) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            if !project
                .environments
                .iter()
                .any(|e| e.name == environment_name)
            {
                return Err(PacsError::ProjectNotFound(format!(
                    "Environment '{environment_name}' not found in project '{project_name}'"
                )));
            }
            project.active_environment = Some(environment_name.to_string());
        }
        self.save_project_by_name(project_name)
    }

    /// Deactivates the active environment for a project.
    pub fn deactivate_environment(&mut self, project_name: ProjectName) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            project.active_environment = None;
        }
        self.save_project_by_name(project_name)
    }

    /// Returns the currently active environment name for a project, if any.
    pub fn get_active_environment(
        &self,
        project_name: ProjectName,
    ) -> Result<Option<String>, PacsError> {
        let project = self.get_project(project_name)?;
        Ok(project.active_environment.clone())
    }

    fn expand_command_with_environment(
        &self,
        cmd: &PacsCommand,
        project_name: ProjectName,
        environment: Option<EnvironmentName>,
    ) -> Result<PacsCommand, PacsError> {
        let project = self.get_project(project_name)?;

        let env_values = environment
            .and_then(|name| project.environments.iter().find(|e| e.name == name))
            .map(|e| &e.values);

        if env_values.is_none() {
            return Ok(cmd.clone());
        }

        let mut unresolved = false;
        let mut output = String::with_capacity(cmd.command.len());

        let mut cursor = 0;
        let src = &cmd.command;

        while let Some(open) = src[cursor..].find("{{").map(|i| cursor + i) {
            output.push_str(&src[cursor..open]);

            let key_start = open + 2;
            let Some(close) = src[key_start..].find("}}").map(|i| key_start + i) else {
                output.push_str(&src[open..]);
                cursor = src.len();
                break;
            };

            let key = &src[key_start..close];

            if let Some(value) = env_values
                .and_then(|vals: &std::collections::BTreeMap<String, String>| vals.get(key))
            {
                output.push_str(value);
            } else {
                unresolved = true;
                output.push_str("{{");
                output.push_str(key);
                output.push_str("}}");
            }

            cursor = close + 2;
        }

        output.push_str(&src[cursor..]);

        if unresolved {
            return Ok(cmd.clone());
        }

        Ok(PacsCommand {
            name: cmd.name.clone(),
            command: output,
            cwd: cmd.cwd.clone(),
            tag: cmd.tag.clone(),
        })
    }

    fn execute(cmd: &PacsCommand) -> Result<(), PacsError> {
        if cmd.command.trim().is_empty() {
            return Err(PacsError::CommandNotFound(cmd.name.clone()));
        }

        let cwd = cmd
            .cwd
            .as_ref()
            .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd.command)
            .current_dir(cwd)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(PacsError::CommandFailed(status.code().unwrap_or(-1)))
        }
    }

    #[must_use]
    pub fn suggest_command_names(&self) -> Vec<String> {
        if let Ok(project) = self.get_active_project() {
            project.commands.iter().map(|c| c.name.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns all project names for shell completion.
    #[must_use]
    pub fn suggest_projects(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }

    /// Returns all unique tags for shell completion.
    #[must_use]
    pub fn suggest_tags(&self, project_name: Option<ProjectName>) -> Vec<String> {
        let project = if let Some(name) = project_name {
            self.get_project(name).ok()
        } else {
            self.get_active_project().ok()
        };

        let Some(project) = project else {
            return Vec::new();
        };

        let mut tags: Vec<String> = project
            .commands
            .iter()
            .map(|c| c.tag.clone())
            .filter(|t| !t.is_empty())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Returns all environment names for the active project or specified project.
    #[must_use]
    pub fn suggest_environments(&self, project_name: Option<ProjectName>) -> Vec<String> {
        let project = if let Some(name) = project_name {
            self.get_project(name).ok()
        } else {
            self.get_active_project().ok()
        };

        let Some(project) = project else {
            return Vec::new();
        };

        project
            .environments
            .iter()
            .map(|e| e.name.clone())
            .collect()
    }

    /// Fuzzy search commands by name or content. Returns matches sorted by relevance.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&PacsCommand> {
        let matcher = SkimMatcherV2::default();

        let mut results: Vec<(&PacsCommand, i64)> = self
            .projects
            .iter()
            .flat_map(|p| p.commands.iter())
            .filter_map(|cmd| {
                let score = matcher
                    .fuzzy_match(&cmd.name, query)
                    .unwrap_or(0)
                    .max(matcher.fuzzy_match(&cmd.command, query).unwrap_or(0));

                if score > 0 { Some((cmd, score)) } else { None }
            })
            .collect();

        // Sort descending by score
        results.sort_by_key(|&(_, score)| -score);

        results.into_iter().map(|(cmd, _)| cmd).collect()
    }
}

fn find_command_mut<'a>(
    project: &'a mut Project,
    name: &str,
) -> Result<&'a mut PacsCommand, PacsError> {
    project
        .commands
        .iter_mut()
        .find(|c| c.name == name)
        .ok_or_else(|| PacsError::CommandNotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_pacs() -> Pacs {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("pacs_test_{}_{}", std::process::id(), id));
        if dir.exists() {
            fs::remove_dir_all(&dir).ok();
        }
        Pacs::init_at(dir).unwrap()
    }

    #[test]
    fn test_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "hello".into(),
                command: "echo hello".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        let commands = pacs.list(Some("test"), None).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "hello");

        pacs.delete_command("hello", Some("test")).unwrap();
        let cmds = pacs.list(Some("test"), None).unwrap();
        assert!(cmds.is_empty());

        pacs.delete_project("test").unwrap();
        assert!(!pacs.projects.iter().any(|p| p.name == "test"));
    }

    #[test]
    fn test_duplicate_in_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "build".into(),
                command: "cargo build".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        // Adding same name should fail
        let result = pacs.add_command(
            PacsCommand {
                name: "build".into(),
                command: "cargo build --release".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PacsError::CommandExists(_)));
    }

    #[test]
    fn test_duplicate_in_same_project_blocks() {
        let mut pacs = temp_pacs();
        pacs.init_project("myproject", None).unwrap();
        pacs.add_command(
            PacsCommand {
                name: "deploy".into(),
                command: "echo deploy".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("myproject"),
        )
        .unwrap();
        // Adding to project with same name should fail
        let result = pacs.add_command(
            PacsCommand {
                name: "deploy".into(),
                command: "echo project deploy".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("myproject"),
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PacsError::CommandExists(_)));
    }

    #[test]
    fn test_duplicate_command_name_in_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("proj1", None).unwrap();
        pacs.add_command(
            PacsCommand {
                name: "test".into(),
                command: "cargo test".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("proj1"),
        )
        .unwrap();

        // Adding same name to same project should fail
        let result = pacs.add_command(
            PacsCommand {
                name: "test".into(),
                command: "cargo test --all".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("proj1"),
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PacsError::CommandExists(_)));
    }

    #[test]
    fn test_duplicates_allowed_between_projects() {
        let mut pacs = temp_pacs();
        pacs.init_project("proj1", None).unwrap();
        pacs.init_project("proj2", None).unwrap();

        pacs.add_command(
            PacsCommand {
                name: "run".into(),
                command: "echo proj1".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("proj1"),
        )
        .unwrap();

        // Adding same name to different project should succeed
        pacs.add_command(
            PacsCommand {
                name: "run".into(),
                command: "echo proj2".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("proj2"),
        )
        .unwrap();

        // Verify both exist
        let cmds1 = pacs.list(Some("proj1"), None).unwrap();
        let cmds2 = pacs.list(Some("proj2"), None).unwrap();
        assert_eq!(cmds1.len(), 1);
        assert_eq!(cmds2.len(), 1);
        assert_eq!(cmds1[0].command, "echo proj1");
        assert_eq!(cmds2[0].command, "echo proj2");
    }

    #[test]
    fn test_delete_command_auto() {
        let mut pacs = temp_pacs();
        pacs.init_project("active_proj", None).unwrap();
        pacs.set_active_project("active_proj").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "cmd1".into(),
                command: "echo 1".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("active_proj"),
        )
        .unwrap();

        // Add command to another project
        pacs.init_project("other_proj", None).unwrap();
        pacs.add_command(
            PacsCommand {
                name: "cmd2".into(),
                command: "echo 2".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("other_proj"),
        )
        .unwrap();

        // delete_command_auto should find cmd1 in active project
        pacs.delete_command_auto("cmd1").unwrap();
        assert!(pacs.list(Some("active_proj"), None).unwrap().is_empty());

        // delete_command_auto should NOT find cmd2 (in other project, not active)
        assert!(pacs.delete_command_auto("cmd2").is_err());

        // Deleting non-existent command should fail
        assert!(pacs.delete_command_auto("cmd3").is_err());
    }

    #[test]
    fn test_get_command_auto() {
        let mut pacs = temp_pacs();
        pacs.init_project("proj", None).unwrap();
        pacs.set_active_project("proj").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "proj-cmd".into(),
                command: "echo project".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("proj"),
        )
        .unwrap();

        assert_eq!(
            pacs.get_command_auto("proj-cmd").unwrap().command,
            "echo project"
        );
        assert!(matches!(
            pacs.get_command_auto("nope"),
            Err(PacsError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_update_command_auto() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "cmd".into(),
                command: "old".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        let old = pacs.update_command_auto("cmd", "new".into()).unwrap();
        assert_eq!(old, "old");
        assert_eq!(pacs.get_command_auto("cmd").unwrap().command, "new");
    }

    #[test]
    fn test_rename_command_auto() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "old-name".into(),
                command: "echo test".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        pacs.rename_command_auto("old-name", "new-name").unwrap();
        assert!(matches!(
            pacs.get_command_auto("old-name"),
            Err(PacsError::CommandNotFound(_))
        ));
        assert_eq!(
            pacs.get_command_auto("new-name").unwrap().command,
            "echo test"
        );
    }

    #[test]
    fn test_rename_to_existing_fails() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "a".into(),
                command: "".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();
        pacs.add_command(
            PacsCommand {
                name: "b".into(),
                command: "".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        let result = pacs.rename_command_auto("a", "b");
        assert!(matches!(result, Err(PacsError::CommandExists(_))));
    }

    #[test]
    fn test_run_auto() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "echo-test".into(),
                command: "echo hello".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        // Test would run the command, but we can't easily test output
        // pacs.run("echo-test", None, None).unwrap();
        assert!(matches!(
            pacs.run("nonexistent", None, None),
            Err(PacsError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_active_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("p1", None).unwrap();
        pacs.init_project("p2", None).unwrap();

        assert!(pacs.get_active_project_name().is_err());

        pacs.set_active_project("p1").unwrap();
        assert_eq!(pacs.get_active_project_name().unwrap(), String::from("p1"));

        pacs.clear_active_project().unwrap();
        assert!(pacs.get_active_project_name().is_err());
    }

    #[test]
    fn test_list_by_tag() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();
        pacs.add_command(
            PacsCommand {
                name: "tag1".into(),
                command: "".into(),
                cwd: None,
                tag: "dev".into(),
            },
            Some("test"),
        )
        .unwrap();
        pacs.add_command(
            PacsCommand {
                name: "tag2".into(),
                command: "".into(),
                cwd: None,
                tag: "prod".into(),
            },
            Some("test"),
        )
        .unwrap();

        let all = pacs.list(Some("test"), None).unwrap();
        let dev = all
            .into_iter()
            .filter(|c| c.tag == "dev")
            .collect::<Vec<_>>();
        assert_eq!(dev.len(), 1);
        assert_eq!(dev[0].name, "tag1");
    }

    #[test]
    fn test_add_command_active_project_fallback() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();

        // Add command without specifying project (uses active project)
        pacs.add_command(
            PacsCommand {
                name: "fallback-cmd".into(),
                command: "echo fallback".into(),
                cwd: None,
                tag: "".into(),
            },
            None,
        )
        .unwrap();

        // Verify command was added to active project
        let commands = pacs.list(None, None).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "fallback-cmd");

        // Also works with explicit project name
        pacs.add_command(
            PacsCommand {
                name: "explicit-cmd".into(),
                command: "echo explicit".into(),
                cwd: None,
                tag: "".into(),
            },
            Some("test"),
        )
        .unwrap();

        let commands = pacs.list(Some("test"), None).unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_delete_command_active_project_fallback() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();
        pacs.set_active_project("test").unwrap();

        // Add a command
        pacs.add_command(
            PacsCommand {
                name: "to-delete".into(),
                command: "echo delete me".into(),
                cwd: None,
                tag: "".into(),
            },
            None,
        )
        .unwrap();

        // Delete without specifying project (uses active project)
        pacs.delete_command("to-delete", None).unwrap();

        let commands = pacs.list(None, None).unwrap();
        assert!(commands.is_empty());
    }
}
