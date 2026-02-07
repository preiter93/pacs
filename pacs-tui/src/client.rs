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
}
