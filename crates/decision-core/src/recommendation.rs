use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    Insufficient,
}

impl ConfidenceLevel {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.8 {
            ConfidenceLevel::High
        } else if score >= 0.5 {
            ConfidenceLevel::Medium
        } else if score >= 0.2 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::Insufficient
        }
    }

    pub fn score(&self) -> f32 {
        match self {
            ConfidenceLevel::High => 0.9,
            ConfidenceLevel::Medium => 0.65,
            ConfidenceLevel::Low => 0.35,
            ConfidenceLevel::Insufficient => 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confidence {
    pub level: ConfidenceLevel,
    pub score: f32,
    pub factors: Vec<ConfidenceFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceFactor {
    pub name: String,
    pub contribution: f32,
    pub description: String,
}

impl Confidence {
    pub fn calculate(factors: Vec<ConfidenceFactor>) -> Self {
        let score: f32 =
            factors.iter().map(|f| f.contribution).sum::<f32>() / factors.len().max(1) as f32;
        let level = ConfidenceLevel::from_score(score);

        Self {
            level,
            score,
            factors,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub decision_type: String,
    pub recommendation: String,
    pub confidence: Confidence,
    pub alternatives: Vec<Alternative>,
    pub evidence_summary: Vec<String>,
    pub missing_info: Vec<String>,
    pub risks: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub option: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSchema {
    pub decision_type: String,
    pub recommendation: String,
    pub confidence_level: ConfidenceLevel,
    pub evidence_count: usize,
    pub completeness: f32,
}

impl Recommendation {
    pub fn new(decision_type: &str, recommendation: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            decision_type: decision_type.to_string(),
            recommendation: recommendation.to_string(),
            confidence: Confidence {
                level: ConfidenceLevel::Insufficient,
                score: 0.0,
                factors: Vec::new(),
            },
            alternatives: Vec::new(),
            evidence_summary: Vec::new(),
            missing_info: Vec::new(),
            risks: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_alternative(mut self, alternative: Alternative) -> Self {
        self.alternatives.push(alternative);
        self
    }

    pub fn to_schema(&self) -> RecommendationSchema {
        RecommendationSchema {
            decision_type: self.decision_type.clone(),
            recommendation: self.recommendation.clone(),
            confidence_level: self.confidence.level.clone(),
            evidence_count: self.evidence_summary.len(),
            completeness: self.confidence.score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_levels() {
        assert!(matches!(
            ConfidenceLevel::from_score(0.9),
            ConfidenceLevel::High
        ));
        assert!(matches!(
            ConfidenceLevel::from_score(0.6),
            ConfidenceLevel::Medium
        ));
        assert!(matches!(
            ConfidenceLevel::from_score(0.3),
            ConfidenceLevel::Low
        ));
        assert!(matches!(
            ConfidenceLevel::from_score(0.1),
            ConfidenceLevel::Insufficient
        ));
    }

    #[test]
    fn test_recommendation() {
        let rec = Recommendation::new("procurement", "Choose Vendor A").with_confidence(
            Confidence::calculate(vec![ConfidenceFactor {
                name: "evidence_count".to_string(),
                contribution: 0.6,
                description: "Moderate evidence".to_string(),
            }]),
        );

        assert_eq!(rec.decision_type, "procurement");
        assert!(matches!(rec.confidence.level, ConfidenceLevel::Medium));
    }
}
