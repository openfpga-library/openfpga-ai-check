use async_trait::async_trait;
use chrono::{DateTime, Utc};
use icu::list::{
    ListFormatter,
    options::{ListFormatterOptions, ListLength},
};
use icu::locale::locale;
use octocrab::{Page, models::repos::RepoCommit};
use std::collections::HashSet;

use crate::{
    checks::{
        Check, CheckResult,
        CheckScore::{self, GuaranteeHuman},
    },
    repo::RepoContext,
};

#[derive(Debug)]
enum CommitStatus {
    LlmCoAuthor(String),
    LlmMentioned(String),
    Clear,
}

trait RatioAndStats {
    fn get_ratio(&self) -> f32;
    fn get_stats(&self) -> Vec<String>;
}

impl RatioAndStats for Vec<CommitStatus> {
    fn get_ratio(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }

        let count = self
            .iter()
            .filter(|&status| {
                matches!(
                    status,
                    CommitStatus::LlmCoAuthor(_) | CommitStatus::LlmMentioned(_)
                )
            })
            .count();

        (count as f32) / self.len() as f32
    }

    fn get_stats(&self) -> Vec<String> {
        if self.is_empty() {
            return vec![];
        }

        let coauthors: Vec<String> = self
            .iter()
            .filter_map(|c| match c {
                CommitStatus::LlmCoAuthor(a) => Some(a.to_string()),
                _ => None,
            })
            .collect();

        let mentions: Vec<String> = self
            .iter()
            .filter_map(|c| match c {
                CommitStatus::LlmMentioned(m) => Some(m.to_string()),
                _ => None,
            })
            .collect();

        let list_formatter = ListFormatter::try_new_or(
            locale!("en").into(),
            ListFormatterOptions::default().with_length(ListLength::Wide),
        )
        .expect("locale should be present");

        let mut messages = Vec::with_capacity(2);

        if !coauthors.is_empty() {
            let unique_coauthors: HashSet<_> = coauthors.iter().collect();
            messages.push(format!(
                "Of the last {} commits {} were co-authored by {}",
                self.len(),
                coauthors.len(),
                list_formatter.format(unique_coauthors.iter())
            ));
        }

        if !mentions.is_empty() {
            let unique_mentions: HashSet<_> = mentions.iter().collect();
            messages.push(format!(
                "Of the last {} commit messages {} mention {}",
                self.len(),
                mentions.len(),
                list_formatter.format(unique_mentions.iter())
            ));
        }

        messages
    }
}

const VIBE_CODE_TIME_NANOS: i64 = 1759273200_000_000_000;

pub struct CommitsCheck;

impl CommitsCheck {
    fn run_pre_ai_check(&self, page: &Page<RepoCommit>) -> anyhow::Result<Vec<CheckResult>> {
        let commit_times: Vec<DateTime<Utc>> = page
            .items
            .iter()
            .filter_map(|c| c.commit.author.as_ref().and_then(|a| a.date))
            .collect();

        let vibe_code_start = DateTime::from_timestamp_nanos(VIBE_CODE_TIME_NANOS);

        let all_pre_ai = commit_times.iter().all(|t| t.le(&vibe_code_start));

        if all_pre_ai {
            Ok(vec![CheckResult {
                name: "Pre-AI check".into(),
                score: GuaranteeHuman,
                output: vec![format!("All commits before 2026")],
            }])
        } else {
            Ok(vec![])
        }
    }

    fn run_coauthor_check(&self, page: &Page<RepoCommit>) -> anyhow::Result<Vec<CheckResult>> {
        let commit_statuses: Vec<CommitStatus> = page
            .items
            .iter()
            .map(|commit_item| {
                if let Some(author_name) = &commit_item
                    .commit
                    .author
                    .as_ref()
                    .and_then(|a| Some(a.name.to_string()))
                {
                    match author_name.as_str() {
                        "Claude" | "Gemini" | "Codex" | "OpenAI" | "Cursor Agent" => {
                            return CommitStatus::LlmCoAuthor(author_name.to_string());
                        }
                        _ => {}
                    }
                }

                match &commit_item.commit.message.to_lowercase() {
                    x if x.contains("claude") => {
                        return CommitStatus::LlmMentioned("Claude".into());
                    }
                    x if x.contains("codex") | x.contains("gpt") => {
                        return CommitStatus::LlmMentioned("OpenAI".into());
                    }
                    x if x.contains("gemini") => {
                        return CommitStatus::LlmMentioned("Gemini".into());
                    }
                    x if x.contains("co-pilot") => {
                        return CommitStatus::LlmMentioned("Co-Pilot".into());
                    }
                    x if x.contains("windsurf") => {
                        return CommitStatus::LlmMentioned("Windsurf".into());
                    }
                    x if x.contains("cursoragent") => {
                        return CommitStatus::LlmMentioned("Cursor Agent".into());
                    }
                    _ => {}
                }

                CommitStatus::Clear
            })
            .collect();

        let ratio = commit_statuses.get_ratio();

        if ratio > 0.0 {
            Ok(vec![CheckResult {
                name: "Co-author check".into(),
                score: CheckScore::SuspectedAi(ratio),
                output: commit_statuses.get_stats(),
            }])
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait]
impl Check for CommitsCheck {
    fn name(&self) -> &'static str {
        "CommitsCheck"
    }

    async fn run(&self, ctx: &RepoContext) -> anyhow::Result<Vec<CheckResult>> {
        let page = ctx.repo()?.list_commits().per_page(100).send().await?;

        Ok(vec![
            self.run_coauthor_check(&page)?,
            self.run_pre_ai_check(&page)?,
        ]
        .into_iter()
        .flatten()
        .collect())
    }
}
