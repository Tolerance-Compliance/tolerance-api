//! POA&M (Plan of Action and Milestones) Validation Module
//!
//! This module provides validation logic to determine which NIST 800-171
//! requirements can and cannot be added to a Plan of Action & Milestones.
//!
//! POA&M Rules (based on CMMC and NIST guidelines):
//! - Level 1 foundational requirements generally CANNOT be added to POA&M
//! - Level 2/3 requirements CAN be added to POA&M with proper justification
//! - High-priority security requirements may have restrictions
//! - Requirements must have documented remediation plans

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::scoring::{CmmcLevel, Priority, RequirementScore, ScoringDatabase};

/// POA&M eligibility status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum PoamEligibility {
    /// Can be added to POA&M
    Eligible,
    /// Cannot be added to POA&M
    NotEligible,
    /// Can be added with conditions/restrictions
    Conditional,
}

/// Reason why a requirement cannot be added to POA&M
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum IneligibilityReason {
    /// Foundational requirement - must be implemented immediately
    FoundationalRequirement,
    /// Critical security control - cannot be deferred
    CriticalControl,
    /// Level 1 requirement - no POA&M allowed
    Level1Requirement,
    /// High-priority requirement with no deferral option
    HighPriorityNoDeferral,
}

/// Condition that must be met for POA&M eligibility
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum PoamCondition {
    /// Requires executive approval
    RequiresExecutiveApproval,
    /// Must have documented compensating controls
    RequiresCompensatingControls,
    /// Must have detailed remediation plan
    RequiresDetailedRemediationPlan,
    /// Limited time window (e.g., 180 days)
    TimeRestricted { max_days: u16 },
    /// Requires risk acceptance documentation
    RequiresRiskAcceptance,
}

/// POA&M validation result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PoamValidation {
    /// Requirement identifier
    pub requirement_id: String,
    /// Whether this requirement can be added to POA&M
    pub eligibility: PoamEligibility,
    /// Reason if not eligible
    pub reason: Option<IneligibilityReason>,
    /// Conditions that must be met if conditional
    pub conditions: Vec<PoamCondition>,
    /// Additional guidance or notes
    pub guidance: String,
}

/// POA&M Validator
pub struct PoamValidator {
    scoring_db: ScoringDatabase,
}

impl PoamValidator {
    /// Create a new POA&M validator
    pub fn new(scoring_db: ScoringDatabase) -> Self {
        Self { scoring_db }
    }

    /// Validate whether a requirement can be added to POA&M
    pub fn validate(&self, requirement_id: &str) -> PoamValidation {
        match self.scoring_db.get_score(requirement_id) {
            Some(score) => self.validate_with_score(requirement_id, score),
            None => PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Conditional,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::RequiresRiskAcceptance,
                ],
                guidance: "This requirement is not in the standard CMMC scoring database. \
                          POA&M may be allowed with proper documentation and risk acceptance."
                    .to_string(),
            },
        }
    }

    fn validate_with_score(&self, requirement_id: &str, score: &RequirementScore) -> PoamValidation {
        // Rule 1: Level 1 foundational requirements cannot be deferred
        if score.cmmc_level == CmmcLevel::Level1 && score.is_foundational {
            return PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::NotEligible,
                reason: Some(IneligibilityReason::FoundationalRequirement),
                conditions: vec![],
                guidance: "This is a foundational CMMC Level 1 requirement and must be \
                          implemented immediately. It cannot be added to a POA&M. These \
                          requirements represent basic cyber hygiene practices that are \
                          essential for protecting CUI."
                    .to_string(),
            };
        }

        // Rule 2: Level 1 non-foundational requirements are conditional
        if score.cmmc_level == CmmcLevel::Level1 {
            return PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Conditional,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresExecutiveApproval,
                    PoamCondition::RequiresCompensatingControls,
                    PoamCondition::TimeRestricted { max_days: 90 },
                ],
                guidance: "This is a CMMC Level 1 requirement. While not foundational, it \
                          requires executive approval, documented compensating controls, and \
                          must be remediated within 90 days."
                    .to_string(),
            };
        }

        // Rule 3: High-priority Level 2 requirements need strict conditions
        if score.cmmc_level == CmmcLevel::Level2 && score.priority == Priority::High {
            return PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Conditional,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresCompensatingControls,
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::TimeRestricted { max_days: 180 },
                ],
                guidance: "This is a high-priority CMMC Level 2 requirement. POA&M is allowed \
                          with compensating controls, a detailed remediation plan, and must be \
                          completed within 180 days."
                    .to_string(),
            };
        }

        // Rule 4: Medium/Low priority Level 2 requirements are generally eligible
        if score.cmmc_level == CmmcLevel::Level2 {
            return PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Eligible,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::TimeRestricted { max_days: 365 },
                ],
                guidance: "This CMMC Level 2 requirement can be added to POA&M with a detailed \
                          remediation plan. Standard timeframe is up to 365 days."
                    .to_string(),
            };
        }

        // Rule 5: Level 3 requirements (if present) have similar rules to Level 2
        if score.cmmc_level == CmmcLevel::Level3 {
            return PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Eligible,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::RequiresRiskAcceptance,
                    PoamCondition::TimeRestricted { max_days: 365 },
                ],
                guidance: "This CMMC Level 3 requirement can be added to POA&M with proper \
                          documentation, risk acceptance, and a detailed remediation plan."
                    .to_string(),
            };
        }

        // Default case
        PoamValidation {
            requirement_id: requirement_id.to_string(),
            eligibility: PoamEligibility::Conditional,
            reason: None,
            conditions: vec![
                PoamCondition::RequiresDetailedRemediationPlan,
                PoamCondition::RequiresRiskAcceptance,
            ],
            guidance: "Standard POA&M rules apply. Consult with your CMMC assessor.".to_string(),
        }
    }

    /// Batch validate multiple requirements
    pub fn validate_batch(&self, requirement_ids: &[String]) -> Vec<PoamValidation> {
        requirement_ids
            .iter()
            .map(|id| self.validate(id))
            .collect()
    }

    /// Get all requirements that cannot be added to POA&M
    pub fn get_non_eligible_requirements(&self) -> Vec<String> {
        self.scoring_db
            .get_foundational()
            .into_iter()
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for PoamValidator {
    fn default() -> Self {
        Self::new(ScoringDatabase::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foundational_not_eligible() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.01.01");
        assert_eq!(result.eligibility, PoamEligibility::NotEligible);
        assert!(matches!(
            result.reason,
            Some(IneligibilityReason::FoundationalRequirement)
        ));
    }

    #[test]
    fn test_level1_non_foundational_conditional() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.01.22");
        assert_eq!(result.eligibility, PoamEligibility::Conditional);
        assert!(!result.conditions.is_empty());
    }

    #[test]
    fn test_level2_eligible() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.01.03");
        assert!(matches!(
            result.eligibility,
            PoamEligibility::Eligible | PoamEligibility::Conditional
        ));
    }

    #[test]
    fn test_batch_validation() {
        let validator = PoamValidator::default();
        let ids = vec!["03.01.01".to_string(), "03.01.03".to_string()];
        let results = validator.validate_batch(&ids);
        assert_eq!(results.len(), 2);
    }
}
