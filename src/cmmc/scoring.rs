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
    /// Load from JSON file
    pub fn new() -> Self {
        let json = std::fs::read_to_string("data/cmmc-scoring.json").unwrap_or_else(|e| {
            tracing::warn!("Can't load scoring file: {}", e);
            String::from("{\"requirements\":{}}")
        });

        let file: ScoringFile = serde_json::from_str(&json).unwrap_or_else(|e| {
            tracing::warn!("Can't parse scoring JSON: {}", e);
            ScoringFile {
                requirements: HashMap::new(),
            }
        });

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

    /// Get score for a requirement. Accepts both Rev 2 (`3.5.3`) and Rev 3
    /// (`03.05.03`) identifier forms — the table is keyed by the Rev 3 form.
    pub fn get_score(&self, requirement_id: &str) -> Option<&RequirementScore> {
        if let Some(score) = self.scores.get(requirement_id) {
            return Some(score);
        }
        normalize_requirement_id(requirement_id).and_then(|id| self.scores.get(&id))
    }

    /// Get all requirements for a CMMC level
    pub fn get_by_level(&self, level: CmmcLevel) -> Vec<(&String, &RequirementScore)> {
        self.scores
            .iter()
            .filter(|(_, score)| score.cmmc_level == level)
            .collect()
    }

    /// All requirement identifiers in the scoring table (Rev 3 form).
    pub fn all_requirement_ids(&self) -> Vec<String> {
        self.scores.keys().cloned().collect()
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

/// Normalize a requirement identifier to the zero-padded Rev 3 form
/// (`3.5.3` → `03.05.03`). Returns `None` for anything that is not three
/// numeric dot-separated segments (e.g. 800-172 ids like `3.1.1e`).
pub fn normalize_requirement_id(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut nums = [0u8; 3];
    for (i, p) in parts.iter().enumerate() {
        nums[i] = p.parse::<u8>().ok()?;
    }
    Some(format!("{:02}.{:02}.{:02}", nums[0], nums[1], nums[2]))
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
        let total = db.calculate_total_points(&["03.01.01".into(), "03.01.02".into()]);
        assert_eq!(total, 10); // 5 + 5 per DoD Assessment Methodology Annex A
    }

    #[test]
    fn test_total_weight_matches_sprs_floor() {
        // Annex A: max deduction 313, so the score floor is 110 - 313 = -203.
        let db = ScoringDatabase::new();
        let all: Vec<String> = db.scores.keys().cloned().collect();
        assert_eq!(all.len(), 110);
        assert_eq!(db.calculate_total_points(&all), 313);
    }

    #[test]
    fn test_r2_form_lookup_normalizes() {
        let db = ScoringDatabase::new();
        let r2 = db.get_score("3.5.3").expect("r2 form should resolve");
        let r3 = db.get_score("03.05.03").expect("r3 form should resolve");
        assert_eq!(r2.point_value, r3.point_value);
        assert_eq!(r2.point_value, 5);
        assert!(db.get_score("3.1.1e").is_none()); // 800-172 form is not SPRS
    }
}
