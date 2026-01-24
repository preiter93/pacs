#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct as _};
use std::{fs, path::PathBuf, process::Command};
use thiserror::Error;

/// Defines whether a command belongs to the global scope or a specific project.
#[derive(Debug, Clone, Copy)]
pub enum Scope<'a> {
    Global,
    Project(&'a str),
}

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

/// A saved command that can be executed.
#[derive(Debug, Deserialize, Clone)]
pub struct PacsCommand {
    /// Unique identifier for this command within its scope.
    pub name: String,
    /// The shell command to execute.
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

/// Context values for a named project context.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Context {
    /// Context identifier (e.g., "dev", "stg").
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
    /// Contexts defined for this project.
    #[serde(default)]
    pub contexts: Vec<Context>,
    /// The active context name used to render placeholders for this project.
    #[serde(default)]
    pub active_context: Option<String>,
}

/// Wrapper for global commands to enable proper TOML serialization.
#[derive(Debug, Serialize, Deserialize, Default)]
struct GlobalCommands {
    #[serde(default)]
    commands: Vec<PacsCommand>,
}

/// Configuration stored in config.toml
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    /// The currently active project name.
    #[serde(default)]
    pub active_project: Option<String>,
}

/// Main container managing global commands and projects.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Pacs {
    /// Commands available in all contexts.
    pub global: Vec<PacsCommand>,
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
            fs::write(base.join("global.toml"), "")?;
        }

        let global = Self::load_global(&base)?;
        let projects = Self::load_projects(&projects_dir)?;

        Ok(Self {
            global,
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
    pub fn set_active_project(&self, name: &str) -> Result<(), PacsError> {
        // Verify the project exists
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

    pub fn get_active_project(&self) -> Result<Option<String>, PacsError> {
        let config = self.load_config()?;
        if let Some(name) = config.active_project {
            if self.get_project(&name).is_ok() {
                return Ok(Some(name));
            }
            self.clear_active_project()?;
        }
        Ok(None)
    }

    /// Creates a new project with the given name and optional path.
    pub fn init_project(&mut self, name: &str, path: Option<String>) -> Result<(), PacsError> {
        if self.projects.iter().any(|p| p.name == name) {
            return Err(PacsError::ProjectExists(name.to_string()));
        }

        let project = Project {
            name: name.to_string(),
            path,
            commands: Vec::new(),
            contexts: Vec::new(),
            active_context: None,
        };

        self.save_project(&project)?;
        self.projects.push(project);
        Ok(())
    }

    /// Removes a project and its associated file.
    pub fn delete_project(&mut self, name: &str) -> Result<(), PacsError> {
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

    /// Adds a command to the specified scope.
    /// Returns an error if a command with the same name already exists in:
    /// - The same scope (global or the same project)
    /// - Global scope (when adding to a project)
    /// - The target project (when adding to global)
    pub fn add_command(&mut self, cmd: PacsCommand, scope: Scope<'_>) -> Result<(), PacsError> {
        // Check for duplicates in global scope
        if self.global.iter().any(|c| c.name == cmd.name) {
            return Err(PacsError::CommandExists(cmd.name));
        }

        match scope {
            Scope::Global => {
                self.global.push(cmd);
                self.save_global()?;
            }
            Scope::Project(name) => {
                let project = self.get_project_mut(name)?;
                // Check for duplicates within the project
                if project.commands.iter().any(|c| c.name == cmd.name) {
                    return Err(PacsError::CommandExists(cmd.name));
                }
                project.commands.push(cmd);
                self.save_project_by_name(name)?;
            }
        }
        Ok(())
    }

    /// Removes a command by name from the specified scope.
    pub fn delete_command(&mut self, name: &str, scope: Scope<'_>) -> Result<(), PacsError> {
        match scope {
            Scope::Global => {
                let before = self.global.len();
                self.global.retain(|c| c.name != name);
                if self.global.len() == before {
                    return Err(PacsError::CommandNotFound(name.to_string()));
                }
                self.save_global()?;
            }
            Scope::Project(proj_name) => {
                let project = self.get_project_mut(proj_name)?;
                let before = project.commands.len();
                project.commands.retain(|c| c.name != name);
                if project.commands.len() == before {
                    return Err(PacsError::CommandNotFound(name.to_string()));
                }
                self.save_project_by_name(proj_name)?;
            }
        }
        Ok(())
    }

    /// Updates a command's content, automatically finding which scope it belongs to.
    /// Searches the active project first (if any), then global.
    /// Returns the old command content.
    pub fn update_command_auto(
        &mut self,
        name: &str,
        new_command: String,
    ) -> Result<String, PacsError> {
        // Check active project first
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && project.commands.iter().any(|c| c.name == name)
        {
            let project = self.get_project_mut(&active)?;
            let cmd = project
                .commands
                .iter_mut()
                .find(|c| c.name == name)
                .expect("command exists");
            let old_command = cmd.command.clone();
            cmd.command = new_command;
            self.save_project_by_name(&active)?;
            return Ok(old_command);
        }

        // Check global
        if let Some(cmd) = self.global.iter_mut().find(|c| c.name == name) {
            let old_command = std::mem::replace(&mut cmd.command, new_command);
            self.save_global()?;
            return Ok(old_command);
        }

        Err(PacsError::CommandNotFound(name.to_string()))
    }

    /// Renames a command, automatically finding which scope it belongs to.
    /// Searches the active project first (if any), then global.
    pub fn rename_command_auto(&mut self, old_name: &str, new_name: &str) -> Result<(), PacsError> {
        // Check if new name already exists in global (would conflict)
        if self.global.iter().any(|c| c.name == new_name) {
            return Err(PacsError::CommandExists(new_name.to_string()));
        }

        // Check active project first
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && project.commands.iter().any(|c| c.name == old_name)
        {
            if project.commands.iter().any(|c| c.name == new_name) {
                return Err(PacsError::CommandExists(new_name.to_string()));
            }
            let project = self.get_project_mut(&active)?;
            let cmd = project
                .commands
                .iter_mut()
                .find(|c| c.name == old_name)
                .expect("command exists");
            cmd.name = new_name.to_string();
            self.save_project_by_name(&active)?;
            return Ok(());
        }

        // Check global
        if let Some(cmd) = self.global.iter_mut().find(|c| c.name == old_name) {
            cmd.name = new_name.to_string();
            self.save_global()?;
            return Ok(());
        }

        Err(PacsError::CommandNotFound(old_name.to_string()))
    }

    /// Gets a command's content, automatically finding which scope it belongs to.
    /// Searches the active project first (if any), then global.
    pub fn get_command_auto(&self, name: &str) -> Result<&PacsCommand, PacsError> {
        // Check active project first
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && let Some(cmd) = project.commands.iter().find(|c| c.name == name)
        {
            return Ok(cmd);
        }

        // Check global
        if let Some(cmd) = self.global.iter().find(|c| c.name == name) {
            return Ok(cmd);
        }

        Err(PacsError::CommandNotFound(name.to_string()))
    }

    /// Removes a command by name, automatically finding which scope it belongs to.
    /// Searches the active project first (if any), then global.
    pub fn delete_command_auto(&mut self, name: &str) -> Result<(), PacsError> {
        // Check active project first
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && project.commands.iter().any(|c| c.name == name)
        {
            let project = self.get_project_mut(&active)?;
            project.commands.retain(|c| c.name != name);
            self.save_project_by_name(&active)?;
            return Ok(());
        }

        // Check global
        let before = self.global.len();
        self.global.retain(|c| c.name != name);
        if self.global.len() == before {
            return Err(PacsError::CommandNotFound(name.to_string()));
        }
        self.save_global()?;
        Ok(())
    }

    /// Returns all commands in the specified scope.
    /// For project scope, when a specific context is provided, commands are expanded using it.
    pub fn list_commands(
        &self,
        scope: Scope<'_>,
        context: Option<&str>,
    ) -> Result<Vec<PacsCommand>, PacsError> {
        match scope {
            Scope::Global => {
                let mut cmds: Vec<PacsCommand> = self.global.clone();
                cmds.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(cmds)
            }
            Scope::Project(project_name) => {
                let project = self.get_project(project_name)?;
                let mut cmds: Vec<PacsCommand> = Vec::with_capacity(project.commands.len());

                if let Some(ctx_name) = context {
                    for c in &project.commands {
                        let pc = Pacs::expand_with_context(c, project, ctx_name);
                        cmds.push(pc);
                    }
                } else {
                    for c in &project.commands {
                        cmds.push(self.expand_with_project_context(c, project_name)?);
                    }
                }

                cmds.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(cmds)
            }
        }
    }

    /// Returns commands filtered by tag.
    pub fn list_by_tag(&self, scope: Scope<'_>, tag: &str) -> Result<Vec<PacsCommand>, PacsError> {
        Ok(self
            .list_commands(scope, None)?
            .into_iter()
            .filter(|c| c.tag == tag)
            .collect())
    }

    /// Runs a command, but refuses to run dangerous commands.
    pub fn run(&self, name: &str, scope: Scope<'_>) -> Result<(), PacsError> {
        let cmd = self.get_command(name, scope)?;
        match scope {
            Scope::Global => Self::execute(cmd),
            Scope::Project(project_name) => {
                let rendered = self.expand_with_project_context(cmd, project_name)?;
                Self::execute(&rendered)
            }
        }
    }

    /// Runs a command by name, automatically finding which scope it belongs to.
    /// Searches the active project first (if any), then global.
    pub fn run_auto(&self, name: &str) -> Result<(), PacsError> {
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && let Some(cmd) = project.commands.iter().find(|c| c.name == name)
        {
            let rendered = self.expand_with_project_context(cmd, &active)?;
            return Self::execute(&rendered);
        }
        if let Some(cmd) = self.global.iter().find(|c| c.name == name) {
            return Self::execute(cmd);
        }
        Err(PacsError::CommandNotFound(name.to_string()))
    }

    fn load_global(base: &std::path::Path) -> Result<Vec<PacsCommand>, PacsError> {
        let path = base.join("global.toml");
        if path.exists() && fs::metadata(&path)?.len() > 0 {
            let global: GlobalCommands = toml::from_str(&fs::read_to_string(&path)?)?;
            Ok(global.commands)
        } else {
            Ok(Vec::new())
        }
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

    fn get_command(&self, name: &str, scope: Scope<'_>) -> Result<&PacsCommand, PacsError> {
        match scope {
            Scope::Global => PacsCommand::find_by_name(&self.global, name),
            Scope::Project(proj_name) => {
                PacsCommand::find_by_name(&self.get_project(proj_name)?.commands, name)
            }
        }
    }

    fn get_project_mut(&mut self, name: &str) -> Result<&mut Project, PacsError> {
        self.projects
            .iter_mut()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| PacsError::ProjectNotFound(name.to_string()))
    }

    fn get_project(&self, name: &str) -> Result<&Project, PacsError> {
        self.projects
            .iter()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| PacsError::ProjectNotFound(name.to_string()))
    }

    fn project_path(&self, name: &str) -> PathBuf {
        self.base_dir.join("projects").join(format!("{name}.toml"))
    }

    fn save_global(&self) -> Result<(), PacsError> {
        let mut commands = self.global.clone();
        commands.sort_by(|a, b| a.name.cmp(&b.name));
        let global = GlobalCommands { commands };
        fs::write(
            self.base_dir.join("global.toml"),
            toml::to_string_pretty(&global)?,
        )?;
        Ok(())
    }

    fn save_project(&self, project: &Project) -> Result<(), PacsError> {
        let mut sorted = project.commands.clone();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        let temp = Project {
            name: project.name.clone(),
            path: project.path.clone(),
            commands: sorted,
            contexts: project.contexts.clone(),
            active_context: project.active_context.clone(),
        };
        fs::write(
            self.project_path(&project.name),
            toml::to_string_pretty(&temp)?,
        )?;
        Ok(())
    }

    pub fn save_project_by_name(&self, name: &str) -> Result<(), PacsError> {
        let project = self.get_project(name)?;
        self.save_project(project)
    }

    /// Adds a new empty context to a project.
    pub fn add_context(&mut self, project_name: &str, context_name: &str) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            if project.contexts.iter().any(|c| c.name == context_name) {
                return Err(PacsError::ProjectExists(format!(
                    "Context '{context_name}' already exists in project '{project_name}'"
                )));
            }
            project.contexts.push(Context {
                name: context_name.to_string(),
                values: std::collections::BTreeMap::new(),
            });
        }
        self.save_project_by_name(project_name)
    }

    /// Removes an existing context from a project.
    pub fn remove_context(
        &mut self,
        project_name: &str,
        context_name: &str,
    ) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            if let Some(idx) = project.contexts.iter().position(|c| c.name == context_name) {
                project.contexts.remove(idx);
                // If the removed context was active, deactivate it.
                if project.active_context.as_deref() == Some(context_name) {
                    project.active_context = None;
                }
            } else {
                return Err(PacsError::ProjectNotFound(format!(
                    "Context '{context_name}' not found in project '{project_name}'"
                )));
            }
        }
        self.save_project_by_name(project_name)
    }

    /// Replaces all key/value pairs in a project's context.
    pub fn edit_context_values(
        &mut self,
        project_name: &str,
        context_name: &str,
        values: std::collections::BTreeMap<String, String>,
    ) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            let ctx = project
                .contexts
                .iter_mut()
                .find(|c| c.name == context_name)
                .ok_or_else(|| {
                    PacsError::ProjectNotFound(format!(
                        "Context '{context_name}' not found in project '{project_name}'"
                    ))
                })?;
            ctx.values = values;
        }
        self.save_project_by_name(project_name)
    }

    /// Activates a specific context for a project.
    pub fn activate_context(
        &mut self,
        project_name: &str,
        context_name: &str,
    ) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            if !project.contexts.iter().any(|c| c.name == context_name) {
                return Err(PacsError::ProjectNotFound(format!(
                    "Context '{context_name}' not found in project '{project_name}'"
                )));
            }
            project.active_context = Some(context_name.to_string());
        }
        self.save_project_by_name(project_name)
    }

    /// Deactivates the active context for a project.
    pub fn deactivate_context(&mut self, project_name: &str) -> Result<(), PacsError> {
        {
            let project = self.get_project_mut(project_name)?;
            project.active_context = None;
        }
        self.save_project_by_name(project_name)
    }

    /// Returns the currently active context name for a project, if any.
    pub fn get_active_context(&self, project_name: &str) -> Result<Option<String>, PacsError> {
        let project = self.get_project(project_name)?;
        Ok(project.active_context.clone())
    }

    fn expand_with_context(
        cmd: &PacsCommand,
        project: &Project,
        context_name: &str,
    ) -> PacsCommand {
        let ctx_values = project
            .contexts
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(context_name))
            .map(|c| &c.values);

        if let Some(vals) = ctx_values {
            let src = &cmd.command;
            let mut out = String::with_capacity(src.len());
            let mut cursor = 0;
            let mut unresolved = false;

            while let Some(open_rel) = src[cursor..].find("{{").map(|i| cursor + i) {
                out.push_str(&src[cursor..open_rel]);
                let key_start = open_rel + 2;
                let Some(close_abs) = src[key_start..].find("}}").map(|i| key_start + i) else {
                    out.push_str(&src[open_rel..]);
                    cursor = src.len();
                    break;
                };
                let key = &src[key_start..close_abs];
                if let Some(val) = vals.get(key) {
                    out.push_str(val);
                } else {
                    unresolved = true;
                    out.push_str("{{");
                    out.push_str(key);
                    out.push_str("}}");
                }
                cursor = close_abs + 2;
            }
            out.push_str(&src[cursor..]);

            let command = if unresolved { cmd.command.clone() } else { out };
            PacsCommand {
                name: cmd.name.clone(),
                command,
                cwd: cmd.cwd.clone(),
                tag: cmd.tag.clone(),
            }
        } else {
            cmd.clone()
        }
    }

    /// Helper to  expand placeholders {{key}}.
    fn expand_with_project_context(
        &self,
        cmd: &PacsCommand,
        project_name: &str,
    ) -> Result<PacsCommand, PacsError> {
        let project = self.get_project(project_name)?;

        let active_ctx_name = project.active_context.as_deref();
        let ctx_values = active_ctx_name
            .and_then(|name| project.contexts.iter().find(|c| c.name == name))
            .map(|c| &c.values);

        // No active context: return raw command unchanged
        if ctx_values.is_none() {
            return Ok(PacsCommand {
                name: cmd.name.clone(),
                command: cmd.command.clone(),
                cwd: cmd.cwd.clone(),
                tag: cmd.tag.clone(),
            });
        }

        let mut unresolved = false;
        let mut output = String::with_capacity(cmd.command.len());

        let mut cursor = 0;
        let src = &cmd.command;

        while let Some(open) = src[cursor..].find("{{").map(|i| cursor + i) {
            output.push_str(&src[cursor..open]);

            let key_start = open + 2;
            let Some(close) = src[key_start..].find("}}").map(|i| key_start + i) else {
                // unmatched opening, copy rest verbatim
                output.push_str(&src[open..]);
                cursor = src.len();
                break;
            };

            let key = &src[key_start..close];

            if let Some(value) = ctx_values.and_then(|vals| vals.get(key)) {
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
            return Ok(PacsCommand {
                name: cmd.name.clone(),
                command: cmd.command.clone(),
                cwd: cmd.cwd.clone(),
                tag: cmd.tag.clone(),
            });
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

    pub fn expand_command_auto(&self, name: &str) -> Result<PacsCommand, PacsError> {
        if let Some(active) = self.get_active_project()?
            && let Ok(project) = self.get_project(&active)
            && let Some(cmd) = project.commands.iter().find(|c| c.name == name)
        {
            return self.expand_with_project_context(cmd, &active);
        }
        if let Some(cmd) = self.global.iter().find(|c| c.name == name) {
            return Ok(cmd.clone());
        }
        Err(PacsError::CommandNotFound(name.to_string()))
    }

    /// Returns command names from global and active project for shell completion.
    #[must_use]
    pub fn suggest_command_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.global.iter().map(|c| c.name.clone()).collect();
        if let Ok(Some(active)) = self.get_active_project()
            && let Ok(project) = self.get_project(&active)
        {
            names.extend(project.commands.iter().map(|c| c.name.clone()));
        }
        names
    }

    /// Returns all project names for shell completion.
    #[must_use]
    pub fn suggest_projects(&self) -> Vec<String> {
        self.projects.iter().map(|p| p.name.clone()).collect()
    }

    /// Returns all unique tags for shell completion.
    #[must_use]
    pub fn suggest_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .global
            .iter()
            .chain(self.projects.iter().flat_map(|p| p.commands.iter()))
            .map(|c| c.tag.clone())
            .filter(|t| !t.is_empty())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Fuzzy search commands by name or content, returns matches sorted by relevance.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&PacsCommand> {
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<_> = self
            .global
            .iter()
            .chain(self.projects.iter().flat_map(|p| p.commands.iter()))
            .filter_map(|cmd| {
                let name_score = matcher.fuzzy_match(&cmd.name, query);
                let cmd_score = matcher.fuzzy_match(&cmd.command, query);
                let score = name_score.unwrap_or(0).max(cmd_score.unwrap_or(0));
                if score > 0 { Some((cmd, score)) } else { None }
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(cmd, _)| cmd).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_pacs() -> Pacs {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("pacs_test_{}_{}", std::process::id(), id));
        // Clean up any leftover directory from previous runs
        if dir.exists() {
            fs::remove_dir_all(&dir).ok();
        }
        Pacs::init_at(dir).unwrap()
    }

    #[test]
    fn test_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("test", None).unwrap();

        assert!(pacs.projects.iter().any(|p| p.name == "test"));

        pacs.add_command(
            PacsCommand {
                name: "hello".into(),
                command: "echo hello".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("test"),
        )
        .unwrap();

        let cmds = pacs.list_commands(Scope::Project("test"), None).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "hello");

        pacs.delete_command("hello", Scope::Project("test"))
            .unwrap();
        let cmds = pacs.list_commands(Scope::Project("test"), None).unwrap();
        assert!(cmds.is_empty());

        pacs.delete_project("test").unwrap();
        assert!(!pacs.projects.iter().any(|p| p.name == "test"));
    }

    #[test]
    fn test_duplicate_in_global() {
        let mut pacs = temp_pacs();

        // Add a global command
        pacs.add_command(
            PacsCommand {
                name: "build".into(),
                command: "cargo build".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        // Adding duplicate to global should fail
        let result = pacs.add_command(
            PacsCommand {
                name: "build".into(),
                command: "cargo build --release".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        );
        assert!(matches!(result, Err(PacsError::CommandExists(_))));
    }

    #[test]
    fn test_duplicate_global_blocks_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("myproject", None).unwrap();

        // Add a global command
        pacs.add_command(
            PacsCommand {
                name: "deploy".into(),
                command: "echo deploy".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        // Adding same name to project should fail (global names are reserved)
        let result = pacs.add_command(
            PacsCommand {
                name: "deploy".into(),
                command: "echo project deploy".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("myproject"),
        );
        assert!(matches!(result, Err(PacsError::CommandExists(_))));
    }

    #[test]
    fn test_duplicate_in_same_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("proj1", None).unwrap();

        // Add command to project
        pacs.add_command(
            PacsCommand {
                name: "test".into(),
                command: "cargo test".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("proj1"),
        )
        .unwrap();

        // Adding duplicate to same project should fail
        let result = pacs.add_command(
            PacsCommand {
                name: "test".into(),
                command: "cargo test --all".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("proj1"),
        );
        assert!(matches!(result, Err(PacsError::CommandExists(_))));
    }

    #[test]
    fn test_duplicates_allowed_between_projects() {
        let mut pacs = temp_pacs();
        pacs.init_project("proj1", None).unwrap();
        pacs.init_project("proj2", None).unwrap();

        // Add command to proj1
        pacs.add_command(
            PacsCommand {
                name: "run".into(),
                command: "echo proj1".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("proj1"),
        )
        .unwrap();

        // Adding same name to proj2 should succeed
        pacs.add_command(
            PacsCommand {
                name: "run".into(),
                command: "echo proj2".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("proj2"),
        )
        .unwrap();

        // Verify both exist
        let cmds1 = pacs.list_commands(Scope::Project("proj1"), None).unwrap();
        let cmds2 = pacs.list_commands(Scope::Project("proj2"), None).unwrap();
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

        // Add command to active project
        pacs.add_command(
            PacsCommand {
                name: "cmd1".into(),
                command: "echo 1".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Project("active_proj"),
        )
        .unwrap();

        // Add command to global
        pacs.add_command(
            PacsCommand {
                name: "cmd2".into(),
                command: "echo 2".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        // delete_command_auto should find cmd1 in active project
        pacs.delete_command_auto("cmd1").unwrap();
        assert!(
            pacs.list_commands(Scope::Project("active_proj"), None)
                .unwrap()
                .is_empty()
        );

        // delete_command_auto should find cmd2 in global
        pacs.delete_command_auto("cmd2").unwrap();
        assert!(pacs.list_commands(Scope::Global, None).unwrap().is_empty());

        // Deleting non-existent command should fail
        let result = pacs.delete_command_auto("nonexistent");
        assert!(matches!(result, Err(PacsError::CommandNotFound(_))));
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
            Scope::Project("proj"),
        )
        .unwrap();

        pacs.add_command(
            PacsCommand {
                name: "global-cmd".into(),
                command: "echo global".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        assert_eq!(
            pacs.get_command_auto("proj-cmd").unwrap().command,
            "echo project"
        );
        assert_eq!(
            pacs.get_command_auto("global-cmd").unwrap().command,
            "echo global"
        );
        assert!(matches!(
            pacs.get_command_auto("nope"),
            Err(PacsError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_update_command_auto() {
        let mut pacs = temp_pacs();
        pacs.add_command(
            PacsCommand {
                name: "cmd".into(),
                command: "old".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        let old = pacs.update_command_auto("cmd", "new".into()).unwrap();
        assert_eq!(old, "old");
        assert_eq!(pacs.get_command_auto("cmd").unwrap().command, "new");
    }

    #[test]
    fn test_rename_command_auto() {
        let mut pacs = temp_pacs();
        pacs.add_command(
            PacsCommand {
                name: "old-name".into(),
                command: "echo test".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
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
        pacs.add_command(
            PacsCommand {
                name: "a".into(),
                command: "".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();
        pacs.add_command(
            PacsCommand {
                name: "b".into(),
                command: "".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        let result = pacs.rename_command_auto("a", "b");
        assert!(matches!(result, Err(PacsError::CommandExists(_))));
    }

    #[test]
    fn test_run_auto() {
        let mut pacs = temp_pacs();
        pacs.add_command(
            PacsCommand {
                name: "echo-test".into(),
                command: "echo hello".into(),
                cwd: None,
                tag: "".into(),
            },
            Scope::Global,
        )
        .unwrap();

        pacs.run_auto("echo-test").unwrap();
        assert!(matches!(
            pacs.run_auto("nonexistent"),
            Err(PacsError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_active_project() {
        let mut pacs = temp_pacs();
        pacs.init_project("p1", None).unwrap();
        pacs.init_project("p2", None).unwrap();

        assert!(pacs.get_active_project().unwrap().is_none());

        pacs.set_active_project("p1").unwrap();
        assert_eq!(pacs.get_active_project().unwrap(), Some("p1".into()));

        pacs.clear_active_project().unwrap();
        assert!(pacs.get_active_project().unwrap().is_none());
    }

    #[test]
    fn test_list_by_tag() {
        let mut pacs = temp_pacs();
        pacs.add_command(
            PacsCommand {
                name: "a".into(),
                command: "".into(),
                cwd: None,
                tag: "dev".into(),
            },
            Scope::Global,
        )
        .unwrap();
        pacs.add_command(
            PacsCommand {
                name: "b".into(),
                command: "".into(),
                cwd: None,
                tag: "prod".into(),
            },
            Scope::Global,
        )
        .unwrap();

        let dev = pacs.list_by_tag(Scope::Global, "dev").unwrap();
        assert_eq!(dev.len(), 1);
        assert_eq!(dev[0].name, "a");
    }
}
