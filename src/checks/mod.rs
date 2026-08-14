pub mod commits;
pub mod contributors;
pub mod readme;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::repo::RepoContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub score: CheckScore,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckScore {
    Add(f32),
    Subtract(f32),
    GuaranteeHuman,
}

#[async_trait]
pub trait Check {
    fn name(&self) -> &'static str;
    async fn run(&self, ctx: &RepoContext) -> anyhow::Result<Vec<CheckResult>>;
}

pub fn all_checks() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(readme::ReadmeCheck),
        Box::new(contributors::ContributorsCheck),
        Box::new(commits::CommitsCheck),
    ]
}
