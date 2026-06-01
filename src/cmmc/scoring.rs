//! CMMC Scoring Module
//!
//! Loads CMMC scoring data from `data/cmmc-scoring.json`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// CMMC Level (1, 2, or 3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum CmmcLevel {
    #[serde(rename = "1")]
    Level1,
    #[serde(rename = "2")]
    Level2,
    #[serde(rename = "3")]
    Level3,
}

/// Priority (High, Medium, Low)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Scoring information for a requirement
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequirementScore {
    pub cmmc_level: CmmcLevel,
    pub point_value: u8,
    pub is_foundational: bool,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// JSON file structure
#[derive(Debug, Deserialize)]
struct ScoringFile {
    requirements: HashMap<String, ScoreJson>,
}

#[derive(Debug, Deserialize)]
struct ScoreJson {
    cmmc_level: u8,
    point_value: u8,
    is_foundational: bool,
    priority: String,
    #[serde(default)]
    notes: Option<String>,
}

/// Scoring database
pub struct ScoringDatabase {
    scores: HashMap<String, RequirementScore>,
}

impl ScoringDatabase {
    /// Load scoring data. The file is small (~21 KB) and static, so it is
    /// bundled into the binary at compile time -- no filesystem or KV access.
    pub fn new() -> Self {
        const SCORING_JSON: &str = include_str!("../../data/cmmc-scoring.json");

        let file: ScoringFile = serde_json::from_str(SCORING_JSON)
            .unwrap_or_else(|_| ScoringFile { requirements: HashMap::new() });

        let mut scores = HashMap::new();
        for (id, score_json) in file.requirements {
            if let Ok(score) = Self::parse_score(score_json) {
                scores.insert(id, score);
            }
        }

        Self { scores }
    }

    fn parse_score(json: ScoreJson) -> Result<RequirementScore, ()> {
        let cmmc_level = match json.cmmc_level {
            1 => CmmcLevel::Level1,
            2 => CmmcLevel::Level2,
            3 => CmmcLevel::Level3,
            _ => return Err(()),
        };

        let priority = match json.priority.to_lowercase().as_str() {
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => return Err(()),
        };

        Ok(RequirementScore {
            cmmc_level,
            point_value: json.point_value,
            is_foundational: json.is_foundational,
            priority,
            notes: json.notes,
        })
    }

    /// Get score for a requirement
    pub fn get_score(&self, requirement_id: &str) -> Option<&RequirementScore> {
        self.scores.get(requirement_id)
    }

    /// Get all requirements for a CMMC level
    pub fn get_by_level(&self, level: CmmcLevel) -> Vec<(&String, &RequirementScore)> {
        self.scores
            .iter()
            .filter(|(_, score)| score.cmmc_level == level)
            .collect()
    }

    /// Get all foundational requirements
    pub fn get_foundational(&self) -> Vec<(&String, &RequirementScore)> {
        self.scores
            .iter()
            .filter(|(_, score)| score.is_foundational)
            .collect()
    }

    /// Calculate total points
    pub fn calculate_total_points(&self, requirement_ids: &[String]) -> u32 {
        requirement_ids
            .iter()
            .filter_map(|id| self.get_score(id))
            .map(|score| score.point_value as u32)
            .sum()
    }
}

impl Default for ScoringDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_scoring() {
        let db = ScoringDatabase::new();
        let score = db.get_score("03.01.01").expect("Should have 03.01.01");
        assert_eq!(score.cmmc_level, CmmcLevel::Level1);
        assert!(score.is_foundational);
    }

    #[test]
    fn test_get_by_level() {
        let db = ScoringDatabase::new();
        let level1 = db.get_by_level(CmmcLevel::Level1);
        assert_eq!(level1.len(), 17); // 17 Level 1 practices
    }

    #[test]
    fn test_calculate_points() {
        let db = ScoringDatabase::new();
        let total = db.calculate_total_points(&vec!["03.01.01".into(), "03.01.02".into()]);
        assert_eq!(total, 6); // 3 + 3
    }
}
