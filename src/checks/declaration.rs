use async_trait::async_trait;

use crate::{
    checks::{Check, CheckResult, CheckScore::DeclaredAi},
    repo::RepoContext,
};

/// Reports the AI tools a core's author lists under `declare_ai` in the
/// `updaters.json` of their latest release.
pub struct DeclarationCheck;

#[async_trait]
impl Check for DeclarationCheck {
    fn name(&self) -> &'static str {
        "DeclarationCheck"
    }

    async fn run(&self, ctx: &RepoContext) -> anyhow::Result<Vec<CheckResult>> {
        if ctx.declared_ai.is_empty() {
            return Ok(vec![]);
        }

        Ok(vec![CheckResult {
            name: "Author declares AI use".to_string(),
            score: DeclaredAi,
            output: vec![format!(
                "Author declares AI use: {}",
                ctx.declared_ai.join(", ")
            )],
        }])
    }
}
