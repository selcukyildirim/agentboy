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
    let decision_types = list_decision_types();
    let dt = decision_types.iter().find(|d| d.id == decision_type_id)
        .ok_or_else(|| format!("Decision type {} not found", decision_type_id))?;

    let missing_facts: Vec<String> = dt.required_facts.clone();
    let confidence = if missing_facts.is_empty() { "High" } else { "Low" };

    Ok(serde_json::json!({
        "decision_type_id": decision_type_id,
        "context": {
            "facts": {},
            "missing_facts": missing_facts,
            "evidence": [],
            "confidence": confidence
        }
    }))
}

#[tauri::command]
pub fn get_recommendation(decision_type_id: String, context: serde_json::Value) -> Result<serde_json::Value, String> {
    let facts = context.get("facts")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let missing = context.get("missing_facts")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let has_all_facts = missing == 0 && !facts.is_empty();

    let (action, confidence, reasoning) = if has_all_facts {
        match decision_type_id.as_str() {
            "financial_optimization" => {
                let budget = facts.get("budget_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let actual = facts.get("actual_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let variance = actual - budget;
                if variance > 0.0 {
                    ("Reduce spending by {:.2}".to_string(), "High".to_string(),
                     format!("Actual exceeds budget by {:.2}. Recommend cost reduction measures.", variance))
                } else {
                    ("Maintain current plan".to_string(), "High".to_string(),
                     format!("Under budget by {:.2}. No immediate action needed.", variance.abs()))
                }
            }
            "cost_reduction" => {
                let current = facts.get("current_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
                ("Negotiate with suppliers".to_string(), "Medium".to_string(),
                 format!("Current cost is {:.2}. Recommend reviewing supplier contracts.", current))
            }
            _ => {
                ("Analyze further".to_string(), "Medium".to_string(),
                 "Insufficient data for confident recommendation".to_string())
            }
        }
    } else {
        ("Insufficient data".to_string(), "Low".to_string(),
         format!("{} required facts are still missing.", missing))
    };

    Ok(serde_json::json!({
        "decision_type_id": decision_type_id,
        "recommendation": {
            "action": action,
            "confidence": confidence,
            "alternatives": [],
            "reasoning": reasoning
        }
    }))
}
