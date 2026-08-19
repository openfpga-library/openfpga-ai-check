use async_trait::async_trait;
use octocrab::{Page, models::Contributor};

use crate::{
    checks::{
        Check, CheckResult,
        CheckScore::{self},
    },
    repo::RepoContext,
};

pub struct ContributorsCheck;

impl ContributorsCheck {
    fn run_contributor_check(&self, page: &Page<Contributor>) -> anyhow::Result<Vec<CheckResult>> {
        Ok(page
            .items
            .iter()
            .filter_map(|contributor| {
                if let Some(author_name) = &contributor.author.name {
                    match author_name.as_str() {
                        "Claude" | "Gemini" | "Codex" | "OpenAI" | "Cursor Agent" => {
                            return Some(CheckResult {
                                name: "AI Contributor".to_string(),
                                score: CheckScore::SuspectedAi(0.75),
                                output: vec![format!(
                                    "{} has contributed to the repo",
                                    &author_name
                                )],
                            });
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect())
    }
}

#[async_trait]
impl Check for ContributorsCheck {
    fn name(&self) -> &'static str {
        "ContributorsCheck"
    }

    async fn run(&self, ctx: &RepoContext) -> anyhow::Result<Vec<CheckResult>> {
        let page = ctx.repo()?.list_contributors().per_page(100).send().await?;

        Ok(vec![self.run_contributor_check(&page)?]
            .into_iter()
            .flatten()
            .collect())
    }
}
