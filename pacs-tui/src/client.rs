use std::collections::BTreeMap;

use anyhow::Context;
use anyhow::Result;
use pacs_core::Pacs;

pub struct PacsClient {
    pacs: Pacs,
}

impl PacsClient {
    pub fn new() -> Result<Self> {
        let pacs = Pacs::init_home().context("Failed to initialize pacs")?;
        Ok(Self { pacs })
    }

    pub fn list_projects(&self) -> Vec<String> {
        self.pacs.projects.iter().map(|p| p.name.clone()).collect()
    }

    pub fn list_environments(&self) -> Vec<String> {
        let Ok(environments) = self.pacs.list_environments(None) else {
            return Vec::new();
        };

        environments.iter().map(|e| e.name.clone()).collect()
    }

    pub fn active_project(&self) -> Option<String> {
        self.pacs.get_active_project_name().ok()
    }

    pub fn active_environment(&self) -> Option<String> {
        self.pacs.get_active_environment(None).ok().flatten()
    }

    pub fn set_active_project(&mut self, name: &str) -> Result<()> {
        self.pacs.set_active_project(name)?;
        Ok(())
    }

    pub fn set_active_environment(&mut self, name: &str) -> Result<()> {
        let project = self.pacs.get_active_project_name()?;
        self.pacs.set_active_environment(&project, name)?;
        Ok(())
    }

    pub fn environment_values(&self) -> BTreeMap<String, String> {
        let Ok(project) = self.pacs.get_active_project() else {
            return BTreeMap::new();
        };
        let Some(active_env) = &project.active_environment else {
            return BTreeMap::new();
        };
        project
            .environments
            .iter()
            .find(|e| &e.name == active_env)
            .map(|e| e.values.clone())
            .unwrap_or_default()
    }
}
