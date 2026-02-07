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
}
