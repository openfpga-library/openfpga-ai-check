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
                }
            }
        }

        self.overall_score = score;
    }
}
