//! POA&M (Plan of Action and Milestones) Validation Module
//!
//! Determines which NIST SP 800-171 requirements may be placed on a Plan of
//! Action & Milestones at CMMC Level 2 assessment time, per **32 CFR
//! §170.21(a)(2)** and the NIST SP 800-171 DoD Assessment Methodology v1.2.1
//! (Annex A weights, loaded from `data/cmmc-scoring.json`).
//!
//! The rules:
//! - Only **1-point** requirements may go on a POA&M ...
//! - ... except the excluded one-pointers named by §170.21(a)(2)(iii)
//!   (03.01.20, 03.01.22, 03.10.03, 03.10.04, 03.10.05) and the SSP
//!   (03.12.04), which has no numeric weight — its absence blocks assessment
//!   entirely;
//! - **03.05.03 (MFA)** is never eligible, even partially implemented;
//! - **03.13.11 (FIPS crypto)** is eligible ONLY in its partial state
//!   (encryption deployed but not FIPS-validated — the −3 tier);
//! - everything weighted 3 or 5 must be MET at assessment time;
//! - POA&M items must close out within **180 days** of the assessment.
//!
//! This module is kept in agreement with the platform's other two copies of
//! the model: `tolerance-ssp-service/src/poam/reference.rs` (export-time
//! validator) and `tolerance-dashboard/lib/poam-eligibility.ts` (UI). Keep
//! all three in sync.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::scoring::{RequirementScore, ScoringDatabase, normalize_requirement_id};

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
    /// 3- or 5-point requirement — must be MET at assessment time per
    /// 32 CFR §170.21(a)(2)(ii)
    MustBeMetAtAssessment,
    /// One-pointer explicitly excluded by 32 CFR §170.21(a)(2)(iii)
    ExcludedOnePointer,
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
    /// Eligible only in the partially-implemented state (the −3 tier of a
    /// variable-weight requirement), never from a fully-unimplemented state
    PartialImplementationOnly,
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

/// POA&M close-out window per the CMMC final rule (32 CFR §170.21(a)(4)).
const POAM_CLOSEOUT_DAYS: u16 = 180;

/// One-pointers excluded from POA&M by 32 CFR §170.21(a)(2)(iii), with the
/// assessor-facing rationale. Rev 3 identifier form.
const EXCLUDED_ONE_POINTERS: &[(&str, &str)] = &[
    (
        "03.01.20",
        "CUI boundary protection (external connections) is non-negotiable",
    ),
    (
        "03.01.22",
        "Public information disclosure control for CUI is critical",
    ),
    (
        "03.10.03",
        "Physical escort of visitors in CUI areas must be controlled",
    ),
    (
        "03.10.04",
        "Physical access audit trail for CUI areas is required",
    ),
    (
        "03.10.05",
        "Physical protection of CUI areas is fundamental",
    ),
];

/// 03.05.03 — MFA. Never POA&M-eligible, even partially implemented.
const MFA_ID: &str = "03.05.03";
/// 03.13.11 — FIPS-validated cryptography. Eligible only in the partial tier.
const FIPS_CRYPTO_ID: &str = "03.13.11";
/// 03.12.04 — the SSP. Unscored; its absence blocks assessment entirely.
const SSP_ID: &str = "03.12.04";

/// POA&M Validator
pub struct PoamValidator {
    scoring_db: ScoringDatabase,
}

impl PoamValidator {
    /// Create a new POA&M validator
    pub fn new(scoring_db: ScoringDatabase) -> Self {
        Self { scoring_db }
    }

    /// Validate whether a requirement can be added to POA&M. Accepts both
    /// Rev 2 (`3.5.3`) and Rev 3 (`03.05.03`) identifier forms.
    pub fn validate(&self, requirement_id: &str) -> PoamValidation {
        let id =
            normalize_requirement_id(requirement_id).unwrap_or_else(|| requirement_id.to_string());
        match self.scoring_db.get_score(&id) {
            Some(score) => self.validate_with_score(requirement_id, &id, score),
            None => PoamValidation {
                requirement_id: requirement_id.to_string(),
                eligibility: PoamEligibility::Conditional,
                reason: None,
                conditions: vec![
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::RequiresRiskAcceptance,
                ],
                guidance: "This requirement is outside SPRS scoring scope (DoD Assessment \
                          Methodology Annex A covers the 110 Rev 2 requirements; Rev-3-only \
                          additions and SP 800-172 enhanced requirements carry no methodology \
                          weight). 32 CFR §170.21(a)(2) does not define POA&M treatment for it — \
                          consult your assessor."
                    .to_string(),
            },
        }
    }

    fn validate_with_score(
        &self,
        original_id: &str,
        normalized_id: &str,
        score: &RequirementScore,
    ) -> PoamValidation {
        // Rule 1: MFA is never POA&M-eligible, even partially implemented.
        if normalized_id == MFA_ID {
            return PoamValidation {
                requirement_id: original_id.to_string(),
                eligibility: PoamEligibility::NotEligible,
                reason: Some(IneligibilityReason::CriticalControl),
                conditions: vec![],
                guidance: "Multifactor authentication (3.5.3) is never POA&M-eligible under \
                          32 CFR §170.21(a)(2), even when partially implemented. It must be \
                          fully implemented at assessment time."
                    .to_string(),
            };
        }

        // Rule 2: FIPS crypto is eligible only in its partial (−3) tier.
        if normalized_id == FIPS_CRYPTO_ID {
            return PoamValidation {
                requirement_id: original_id.to_string(),
                eligibility: PoamEligibility::Conditional,
                reason: None,
                conditions: vec![
                    PoamCondition::PartialImplementationOnly,
                    PoamCondition::RequiresDetailedRemediationPlan,
                    PoamCondition::TimeRestricted {
                        max_days: POAM_CLOSEOUT_DAYS,
                    },
                ],
                guidance: "FIPS-validated cryptography (3.13.11) may be deferred on a POA&M \
                          ONLY when encryption is already deployed but not yet FIPS-validated \
                          (the −3 partial tier). With no CUI encryption at all (−5), it cannot \
                          be deferred — deploy encryption first. Close-out within 180 days."
                    .to_string(),
            };
        }

        // Rule 3: the SSP has no numeric weight; without it assessment cannot
        // proceed, so it can never be deferred.
        if normalized_id == SSP_ID {
            return PoamValidation {
                requirement_id: original_id.to_string(),
                eligibility: PoamEligibility::NotEligible,
                reason: Some(IneligibilityReason::CriticalControl),
                conditions: vec![],
                guidance: "The System Security Plan (3.12.4) is the foundation of the \
                          assessment — the assessment cannot proceed without it, so it can \
                          never be placed on a POA&M."
                    .to_string(),
            };
        }

        // Rule 4: one-pointers named by 32 CFR §170.21(a)(2)(iii) are excluded.
        if let Some((_, reason)) = EXCLUDED_ONE_POINTERS
            .iter()
            .find(|(id, _)| *id == normalized_id)
        {
            return PoamValidation {
                requirement_id: original_id.to_string(),
                eligibility: PoamEligibility::NotEligible,
                reason: Some(IneligibilityReason::ExcludedOnePointer),
                conditions: vec![],
                guidance: format!(
                    "Excluded from POA&M by 32 CFR §170.21(a)(2)(iii) despite its 1-point \
                     weight: {reason}. It must be fully implemented at assessment time."
                ),
            };
        }

        // Rule 5: 3- and 5-point requirements must be MET at assessment time.
        if score.point_value > 1 {
            return PoamValidation {
                requirement_id: original_id.to_string(),
                eligibility: PoamEligibility::NotEligible,
                reason: Some(IneligibilityReason::MustBeMetAtAssessment),
                conditions: vec![],
                guidance: format!(
                    "This is a {}-point requirement under the DoD Assessment Methodology \
                     (Annex A). 32 CFR §170.21(a)(2)(ii) limits POA&M items to 1-point \
                     requirements, so it must be fully implemented at assessment time.",
                    score.point_value
                ),
            };
        }

        // Rule 6: remaining 1-pointers are eligible with a remediation plan
        // and the 180-day close-out window.
        PoamValidation {
            requirement_id: original_id.to_string(),
            eligibility: PoamEligibility::Eligible,
            reason: None,
            conditions: vec![
                PoamCondition::RequiresDetailedRemediationPlan,
                PoamCondition::TimeRestricted {
                    max_days: POAM_CLOSEOUT_DAYS,
                },
            ],
            guidance: "1-point requirement, POA&M-eligible per 32 CFR §170.21(a)(2). A \
                      detailed remediation plan is required and the POA&M must be closed out \
                      within 180 days of the assessment."
                .to_string(),
        }
    }

    /// Batch validate multiple requirements
    pub fn validate_batch(&self, requirement_ids: &[String]) -> Vec<PoamValidation> {
        requirement_ids.iter().map(|id| self.validate(id)).collect()
    }

    /// Get all requirements that can never be added to POA&M: every 3- and
    /// 5-point requirement, the excluded one-pointers, MFA, and the SSP.
    pub fn get_non_eligible_requirements(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .scoring_db
            .all_requirement_ids()
            .into_iter()
            .filter(|id| self.validate(id).eligibility == PoamEligibility::NotEligible)
            .collect();
        ids.sort();
        ids
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
    fn mfa_never_eligible_in_both_id_forms() {
        let validator = PoamValidator::default();
        for id in ["03.05.03", "3.5.3"] {
            let result = validator.validate(id);
            assert_eq!(result.eligibility, PoamEligibility::NotEligible, "{id}");
            assert!(matches!(
                result.reason,
                Some(IneligibilityReason::CriticalControl)
            ));
        }
    }

    #[test]
    fn fips_crypto_partial_only() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.13.11");
        assert_eq!(result.eligibility, PoamEligibility::Conditional);
        assert!(
            result
                .conditions
                .iter()
                .any(|c| matches!(c, PoamCondition::PartialImplementationOnly))
        );
    }

    #[test]
    fn ssp_never_eligible() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.12.04");
        assert_eq!(result.eligibility, PoamEligibility::NotEligible);
    }

    #[test]
    fn excluded_one_pointers_not_eligible() {
        let validator = PoamValidator::default();
        for id in ["03.01.20", "03.01.22", "03.10.03", "03.10.04", "03.10.05"] {
            let result = validator.validate(id);
            assert_eq!(result.eligibility, PoamEligibility::NotEligible, "{id}");
            assert!(matches!(
                result.reason,
                Some(IneligibilityReason::ExcludedOnePointer)
            ));
        }
    }

    #[test]
    fn heavy_requirements_must_be_met() {
        let validator = PoamValidator::default();
        // 5-point and 3-point requirements cannot be deferred.
        for id in ["03.01.01", "03.09.01"] {
            let result = validator.validate(id);
            assert_eq!(result.eligibility, PoamEligibility::NotEligible, "{id}");
            assert!(matches!(
                result.reason,
                Some(IneligibilityReason::MustBeMetAtAssessment)
            ));
        }
    }

    #[test]
    fn plain_one_pointer_eligible() {
        let validator = PoamValidator::default();
        let result = validator.validate("03.01.03");
        assert_eq!(result.eligibility, PoamEligibility::Eligible);
        assert!(
            result
                .conditions
                .iter()
                .any(|c| matches!(c, PoamCondition::TimeRestricted { max_days: 180 }))
        );
    }

    #[test]
    fn out_of_scope_is_conditional() {
        let validator = PoamValidator::default();
        // Rev-3-only family and an 800-172 enhanced-requirement form.
        for id in ["03.15.01", "3.1.1e"] {
            let result = validator.validate(id);
            assert_eq!(result.eligibility, PoamEligibility::Conditional, "{id}");
        }
    }

    #[test]
    fn non_eligible_list_matches_cfr() {
        let validator = PoamValidator::default();
        let ids = validator.get_non_eligible_requirements();
        for expected in ["03.05.03", "03.12.04", "03.01.20", "03.01.01"] {
            assert!(ids.contains(&expected.to_string()), "missing {expected}");
        }
        assert!(!ids.contains(&"03.01.03".to_string())); // plain 1-pointer
        assert!(!ids.contains(&"03.13.11".to_string())); // conditional, not never
        // 44 five-pointers + 14 three-pointers + 5 excluded one-pointers + the
        // SSP, minus 03.13.11 (a five-pointer that is Conditional, not never-
        // eligible). MFA (03.05.03) is already counted among the five-pointers.
        assert_eq!(ids.len(), 63);
    }

    #[test]
    fn batch_validation() {
        let validator = PoamValidator::default();
        let ids = vec!["03.01.01".to_string(), "03.01.03".to_string()];
        let results = validator.validate_batch(&ids);
        assert_eq!(results.len(), 2);
    }
}
