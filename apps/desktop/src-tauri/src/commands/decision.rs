use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionType {
    pub id: String,
    pub name: String,
    pub category: String,
    pub required_facts: Vec<String>,
    pub risk_level: String,
}

#[tauri::command]
pub fn list_decision_types() -> Vec<DecisionType> {
    vec![
        DecisionType {
            id: "financial_optimization".to_string(),
            name: "Financial Optimization".to_string(),
            category: "Finance".to_string(),
            required_facts: vec!["budget_amount".to_string(), "actual_amount".to_string()],
            risk_level: "Medium".to_string(),
        },
        DecisionType {
            id: "risk_assessment".to_string(),
            name: "Risk Assessment".to_string(),
            category: "Risk".to_string(),
            required_facts: vec!["risk_factors".to_string(), "mitigation_options".to_string()],
            risk_level: "High".to_string(),
        },
        DecisionType {
            id: "cost_reduction".to_string(),
            name: "Cost Reduction".to_string(),
            category: "Procurement".to_string(),
            required_facts: vec!["current_cost".to_string(), "alternative_options".to_string()],
            risk_level: "Low".to_string(),
        },
    ]
}

#[tauri::command]
pub fn get_decision_context(decision_type_id: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "decision_type_id": decision_type_id,
        "context": {
            "facts": {},
            "missing_facts": [],
            "evidence": [],
            "confidence": "Low"
        }
    }))
}

#[tauri::command]
pub fn get_recommendation(decision_type_id: String, context: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "decision_type_id": decision_type_id,
        "recommendation": {
            "action": "Pending analysis",
            "confidence": "Low",
            "alternatives": [],
            "reasoning": "Insufficient data for confident recommendation"
        }
    }))
}