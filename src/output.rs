use std::collections::HashMap;

use crate::checks::{CheckResult, CheckScore};
use chrono::{DateTime, Utc, serde::ts_milliseconds};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreResultsOutputFile {
    #[serde(with = "ts_milliseconds")]
    pub last_run: DateTime<Utc>,
    pub results: HashMap<String, Vec<CheckResult>>,
    pub overall_score: f32,
}

impl CoreResultsOutputFile {
    pub fn calculate_overall_score(&mut self) {
        let declared = self
            .results
            .values()
            .flatten()
            .any(|result| matches!(result.score, CheckScore::DeclaredAi));

        // The author's own word beats anything the other checks infer,
        // including GuaranteeHuman.
        if declared {
            self.overall_score = 1.0;
            return;
        }

        let mut score: f32 = 0.0;

        for (_check, results) in &self.results {
            for result in results {
                match result.score {
                    CheckScore::SuspectedAi(value) => {
                        score = score.max(value);
                    }
                    CheckScore::GuaranteeHuman => {
                        self.overall_score = 0.0;
                        return;
                    }
                    CheckScore::DeclaredAi => {}
                }
            }
        }

        self.overall_score = score;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(score: CheckScore) -> CheckResult {
        CheckResult {
            name: String::new(),
            score,
            output: vec![],
        }
    }

    fn overall_score(results: Vec<(&str, CheckScore)>) -> f32 {
        let mut output = CoreResultsOutputFile {
            last_run: Utc::now(),
            results: results
                .into_iter()
                .map(|(check, score)| (check.to_string(), vec![result(score)]))
                .collect(),
            overall_score: 0.0,
        };

        output.calculate_overall_score();
        output.overall_score
    }

    #[test]
    fn declared_ai_overrides_guarantee_human() {
        let score = overall_score(vec![
            ("CommitsCheck", CheckScore::GuaranteeHuman),
            ("DeclarationCheck", CheckScore::DeclaredAi),
            ("ReadmeCheck", CheckScore::SuspectedAi(0.5)),
        ]);

        assert_eq!(score, 1.0);
    }
}
