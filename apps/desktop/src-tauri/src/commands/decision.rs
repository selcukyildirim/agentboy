use decision_core::context::{DecisionContextBuilder, MissingInfoDetector};
use decision_core::decision_type::DecisionType;
use decision_core::memory::DecisionMemory;
use decision_core::recommendation::{Confidence, ConfidenceFactor, Recommendation};
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Serialize)]
pub struct DecisionTypeInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub required_facts: Vec<String>,
    pub risk_level: String,
}

fn registry() -> decision_core::decision_type::DecisionTypeRegistry {
    decision_core::decision_type::DecisionTypeRegistry::new()
}

fn parse_type(id: &str) -> DecisionType {
    match id {
        "Procurement" => DecisionType::Procurement,
        "BudgetAllocation" => DecisionType::BudgetAllocation,
        "VendorSelection" => DecisionType::VendorSelection,
        "Hiring" => DecisionType::Hiring,
        "Pricing" => DecisionType::Pricing,
        "Inventory" => DecisionType::Inventory,
        "Investment" => DecisionType::Investment,
        "RiskAssessment" => DecisionType::RiskAssessment,
        other => DecisionType::Custom(other.to_string()),
    }
}

fn humanize(dt: &DecisionType) -> String {
    match dt {
        DecisionType::Procurement => "Procurement",
        DecisionType::BudgetAllocation => "Budget Allocation",
        DecisionType::VendorSelection => "Vendor Selection",
        DecisionType::Hiring => "Hiring",
        DecisionType::Pricing => "Pricing",
        DecisionType::Inventory => "Inventory",
        DecisionType::Investment => "Investment",
        DecisionType::RiskAssessment => "Risk Assessment",
        DecisionType::Custom(name) => name.as_str(),
    }
    .to_string()
}

fn category_for(dt: &DecisionType) -> &'static str {
    match dt {
        DecisionType::Procurement | DecisionType::VendorSelection => "Procurement",
        DecisionType::BudgetAllocation | DecisionType::Investment => "Finance",
        DecisionType::Hiring => "HR",
        DecisionType::Pricing => "Sales",
        DecisionType::Inventory => "Operations",
        DecisionType::RiskAssessment => "Risk",
        DecisionType::Custom(_) => "Custom",
    }
}

fn risk_for(dt: &DecisionType) -> &'static str {
    match dt {
        DecisionType::VendorSelection | DecisionType::Investment | DecisionType::RiskAssessment => {
            "High"
        }
        DecisionType::Inventory => "Low",
        _ => "Medium",
    }
}

fn confidence_label(score: f32) -> &'static str {
    if score >= 0.75 {
        "High"
    } else if score >= 0.45 {
        "Medium"
    } else {
        "Low"
    }
}

#[tauri::command]
pub fn list_decision_types() -> Vec<DecisionTypeInfo> {
    let reg = registry();
    let mut types: Vec<DecisionType> = reg.list_types().into_iter().cloned().collect();
    types.sort_by_key(|t| t.to_string());

    types
        .iter()
        .map(|t| DecisionTypeInfo {
            id: t.to_string(),
            name: humanize(t),
            category: category_for(t).to_string(),
            required_facts: reg
                .get_facts(t)
                .map(|facts| facts.iter().map(|f| f.key.clone()).collect())
                .unwrap_or_default(),
            risk_level: risk_for(t).to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn get_decision_context(decision_type_id: String) -> Result<serde_json::Value, String> {
    let dt = parse_type(&decision_type_id);
    let ctx = DecisionContextBuilder::new(dt).build();
    let report = MissingInfoDetector::new().detect(&ctx);

    Ok(serde_json::json!({
        "decision_type_id": decision_type_id,
        "context": {
            "missing_facts": report.missing_required.iter().map(|m| m.key.clone()).collect::<Vec<_>>(),
            "missing_optional": report.missing_optional.iter().map(|m| m.key.clone()).collect::<Vec<_>>(),
            "can_proceed": report.can_proceed,
            "completeness_score": ctx.completeness_score,
            "confidence": confidence_label(ctx.completeness_score),
        }
    }))
}

#[tauri::command]
pub async fn get_recommendation(
    state: tauri::State<'_, AppState>,
    decision_type_id: String,
    context: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let dt = parse_type(&decision_type_id);

    let mut builder = DecisionContextBuilder::new(dt.clone());
    if let Some(facts) = context
        .get("facts")
        .and_then(|v| v.as_object())
        .or_else(|| context.as_object())
    {
        for (key, value) in facts {
            builder.add_fact(key, value.clone(), "user", 1.0);
        }
    }

    let ctx = builder.build();
    let report = MissingInfoDetector::new().detect(&ctx);
    let confidence = Confidence::calculate(vec![ConfidenceFactor {
        name: "completeness".to_string(),
        contribution: ctx.completeness_score,
        description: "Provided facts completeness".to_string(),
    }]);

    let recommendation_text = if report.can_proceed {
        format!(
            "Proceed with the {} decision; required facts are available.",
            humanize(&dt)
        )
    } else {
        format!(
            "Insufficient data for the {} decision. Collect the missing facts first.",
            humanize(&dt)
        )
    };

    let rec = Recommendation {
        id: uuid::Uuid::new_v4().to_string(),
        decision_type: dt.to_string(),
        recommendation: recommendation_text,
        confidence,
        alternatives: vec![],
        evidence_summary: vec![],
        missing_info: report
            .missing_required
            .iter()
            .map(|m| m.key.clone())
            .collect(),
        risks: vec![],
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Persist to local decision memory (decision-core).
    let memory = DecisionMemory::new(state.pool.clone());
    if let Err(e) = memory.save(&ctx, &rec).await {
        tracing::warn!(error = %e, "Failed to save decision memory");
    }

    serde_json::to_value(serde_json::json!({ "recommendation": rec })).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_decision_history(
    state: tauri::State<'_, AppState>,
    decision_type_id: Option<String>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(50) as i64;

    let rows: Vec<(String, String, String, String)> = match decision_type_id {
        Some(id) => sqlx::query_as(
            "SELECT id, decision_type, recommendation_json, created_at
             FROM decision_memory WHERE decision_type = ?
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?,
        None => sqlx::query_as(
            "SELECT id, decision_type, recommendation_json, created_at
             FROM decision_memory ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?,
    };

    let records: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, dtype, rec, created_at)| {
            serde_json::json!({
                "id": id,
                "decision_type": dtype,
                "recommendation": serde_json::from_str::<serde_json::Value>(&rec).unwrap_or(serde_json::Value::Null),
                "created_at": created_at,
            })
        })
        .collect();

    Ok(serde_json::json!({ "records": records }))
}
