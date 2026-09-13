use crate::decision_type::{DecisionType, RequiredFact};
use crate::evidence::Evidence;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub decision_type: DecisionType,
    pub facts: HashMap<String, Fact>,
    pub evidences: Vec<Evidence>,
    pub missing_facts: Vec<String>,
    pub completeness_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: String,
    pub value: serde_json::Value,
    pub source: String,
    pub confidence: f32,
}

pub struct DecisionContextBuilder {
    decision_type: DecisionType,
    facts: HashMap<String, Fact>,
    evidences: Vec<Evidence>,
}

impl DecisionContextBuilder {
    pub fn new(decision_type: DecisionType) -> Self {
        Self {
            decision_type,
            facts: HashMap::new(),
            evidences: Vec::new(),
        }
    }

    pub fn add_fact(&mut self, key: &str, value: serde_json::Value, source: &str, confidence: f32) {
        self.facts.insert(
            key.to_string(),
            Fact {
                key: key.to_string(),
                value,
                source: source.to_string(),
                confidence,
            },
        );
    }

    pub fn add_evidence(&mut self, evidence: Evidence) {
        self.evidences.push(evidence);
    }

    pub fn build(&self) -> DecisionContext {
        let required_facts = self.decision_type.required_facts();
        let missing = self.detect_missing(&required_facts);
        let completeness = self.calculate_completeness(&required_facts);

        DecisionContext {
            decision_type: self.decision_type.clone(),
            facts: self.facts.clone(),
            evidences: self.evidences.clone(),
            missing_facts: missing,
            completeness_score: completeness,
        }
    }

    fn detect_missing(&self, required_facts: &[RequiredFact]) -> Vec<String> {
        required_facts
            .iter()
            .filter(|f| f.required && !self.facts.contains_key(&f.key))
            .map(|f| f.key.clone())
            .collect()
    }

    fn calculate_completeness(&self, required_facts: &[RequiredFact]) -> f32 {
        if required_facts.is_empty() {
            return 1.0;
        }
        let present = required_facts
            .iter()
            .filter(|f| self.facts.contains_key(&f.key))
            .count();
        present as f32 / required_facts.len() as f32
    }
}

pub struct MissingInfoDetector;

impl MissingInfoDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, context: &DecisionContext) -> MissingInfoReport {
        let required_facts = context.decision_type.required_facts();
        let mut missing_required = Vec::new();
        let mut missing_optional = Vec::new();

        for fact in &required_facts {
            if !context.facts.contains_key(&fact.key) {
                if fact.required {
                    missing_required.push(MissingFact {
                        key: fact.key.clone(),
                        description: fact.description.clone(),
                        impact: ImpactLevel::High,
                    });
                } else {
                    missing_optional.push(MissingFact {
                        key: fact.key.clone(),
                        description: fact.description.clone(),
                        impact: ImpactLevel::Medium,
                    });
                }
            }
        }

        let can_proceed = missing_required.is_empty();
        let total_required = required_facts.iter().filter(|f| f.required).count();
        let total_missing_required = missing_required.len();

        MissingInfoReport {
            missing_required,
            missing_optional,
            can_proceed,
            total_required,
            total_missing_required,
        }
    }
}

impl Default for MissingInfoDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingInfoReport {
    pub missing_required: Vec<MissingFact>,
    pub missing_optional: Vec<MissingFact>,
    pub can_proceed: bool,
    pub total_required: usize,
    pub total_missing_required: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingFact {
    pub key: String,
    pub description: String,
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let mut builder = DecisionContextBuilder::new(DecisionType::Procurement);
        builder.add_fact("vendor_quotes", serde_json::json!([{"vendor": "A", "price": 100}]), "doc1", 0.9);
        builder.add_fact("budget_limit", serde_json::json!(500), "user", 1.0);

        let context = builder.build();
        assert!(context.completeness_score > 0.0);
        assert!(context.facts.len() == 2);
    }

    #[test]
    fn test_missing_info_detector() {
        let mut builder = DecisionContextBuilder::new(DecisionType::Procurement);
        builder.add_fact("vendor_quotes", serde_json::json!([{"vendor": "A"}]), "doc1", 0.9);
        builder.add_fact("budget_limit", serde_json::json!(500), "user", 1.0);

        let context = builder.build();
        let detector = MissingInfoDetector::new();
        let report = detector.detect(&context);

        assert!(report.missing_required.is_empty());
        assert!(report.can_proceed);
    }
}