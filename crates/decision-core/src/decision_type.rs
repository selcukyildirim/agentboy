use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DecisionType {
    Procurement,
    BudgetAllocation,
    VendorSelection,
    Hiring,
    Pricing,
    Inventory,
    Investment,
    RiskAssessment,
    Custom(String),
}

impl fmt::Display for DecisionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionType::Procurement => write!(f, "Procurement"),
            DecisionType::BudgetAllocation => write!(f, "BudgetAllocation"),
            DecisionType::VendorSelection => write!(f, "VendorSelection"),
            DecisionType::Hiring => write!(f, "Hiring"),
            DecisionType::Pricing => write!(f, "Pricing"),
            DecisionType::Inventory => write!(f, "Inventory"),
            DecisionType::Investment => write!(f, "Investment"),
            DecisionType::RiskAssessment => write!(f, "RiskAssessment"),
            DecisionType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

impl DecisionType {
    pub fn required_facts(&self) -> Vec<RequiredFact> {
        match self {
            DecisionType::Procurement => vec![
                RequiredFact::new("vendor_quotes", true, "Vendor price quotes"),
                RequiredFact::new("historical_prices", false, "Historical price data"),
                RequiredFact::new("budget_limit", true, "Budget constraint"),
                RequiredFact::new("policy_doc", false, "Procurement policy"),
                RequiredFact::new("delivery_timeline", false, "Expected delivery dates"),
            ],
            DecisionType::BudgetAllocation => vec![
                RequiredFact::new("current_budget", true, "Current budget status"),
                RequiredFact::new("department_needs", true, "Department requirements"),
                RequiredFact::new("historical_spend", false, "Past spending patterns"),
                RequiredFact::new("revenue_forecast", false, "Revenue projections"),
            ],
            DecisionType::VendorSelection => vec![
                RequiredFact::new("vendor_profiles", true, "Vendor capability profiles"),
                RequiredFact::new("pricing", true, "Pricing information"),
                RequiredFact::new("references", false, "Customer references"),
                RequiredFact::new("compliance", false, "Compliance certifications"),
            ],
            DecisionType::Hiring => vec![
                RequiredFact::new("job_description", true, "Job requirements"),
                RequiredFact::new("budget", true, "Hiring budget"),
                RequiredFact::new("team_composition", false, "Current team structure"),
                RequiredFact::new("market_rates", false, "Market salary data"),
            ],
            DecisionType::Pricing => vec![
                RequiredFact::new("cost_structure", true, "Product cost data"),
                RequiredFact::new("competitor_prices", false, "Competitor pricing"),
                RequiredFact::new("demand_data", false, "Demand signals"),
                RequiredFact::new("margin_target", true, "Target margin"),
            ],
            DecisionType::Inventory => vec![
                RequiredFact::new("current_stock", true, "Current inventory levels"),
                RequiredFact::new("sales_forecast", false, "Sales predictions"),
                RequiredFact::new("lead_times", true, "Supplier lead times"),
                RequiredFact::new("storage_capacity", false, "Warehouse capacity"),
            ],
            DecisionType::Investment => vec![
                RequiredFact::new("investment_options", true, "Available investment options"),
                RequiredFact::new("risk_profile", true, "Risk tolerance"),
                RequiredFact::new("expected_returns", false, "Return projections"),
                RequiredFact::new("liquidity_needs", false, "Cash flow requirements"),
            ],
            DecisionType::RiskAssessment => vec![
                RequiredFact::new("risk_factors", true, "Identified risk factors"),
                RequiredFact::new("mitigation_options", false, "Available mitigations"),
                RequiredFact::new("impact_analysis", true, "Potential impact data"),
                RequiredFact::new("historical_incidents", false, "Past incident data"),
            ],
            DecisionType::Custom(_) => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFact {
    pub key: String,
    pub required: bool,
    pub description: String,
}

impl RequiredFact {
    pub fn new(key: &str, required: bool, description: &str) -> Self {
        Self {
            key: key.to_string(),
            required,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTypeRegistry {
    types: HashMap<DecisionType, Vec<RequiredFact>>,
}

impl DecisionTypeRegistry {
    pub fn new() -> Self {
        let mut types = HashMap::new();
        for dt in [
            DecisionType::Procurement,
            DecisionType::BudgetAllocation,
            DecisionType::VendorSelection,
            DecisionType::Hiring,
            DecisionType::Pricing,
            DecisionType::Inventory,
            DecisionType::Investment,
            DecisionType::RiskAssessment,
        ] {
            types.insert(dt.clone(), dt.required_facts());
        }
        Self { types }
    }

    pub fn get_facts(&self, decision_type: &DecisionType) -> Option<&Vec<RequiredFact>> {
        self.types.get(decision_type)
    }

    pub fn list_types(&self) -> Vec<&DecisionType> {
        self.types.keys().collect()
    }
}

impl Default for DecisionTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procurement_required_facts() {
        let facts = DecisionType::Procurement.required_facts();
        assert!(facts.len() >= 3);
        assert!(facts.iter().any(|f| f.key == "vendor_quotes" && f.required));
    }

    #[test]
    fn test_registry() {
        let registry = DecisionTypeRegistry::new();
        assert!(registry.list_types().len() >= 8);
        assert!(registry.get_facts(&DecisionType::Procurement).is_some());
    }
}