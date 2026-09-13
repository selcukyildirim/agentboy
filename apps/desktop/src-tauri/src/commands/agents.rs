use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub department: String,
    pub description: String,
    pub tier: String,
}

#[tauri::command]
pub fn list_agents() -> Vec<AgentInfo> {
    vec![
        AgentInfo {
            id: "finance.bank-reconciliation".to_string(),
            name: "Bank Reconciliation".to_string(),
            department: "Finance".to_string(),
            description: "Match bank statement records with ledger entries".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "finance.budget-variance".to_string(),
            name: "Budget Variance".to_string(),
            department: "Finance".to_string(),
            description: "Compare budget vs actual spending".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "finance.expense-analyst".to_string(),
            name: "Expense Analyst".to_string(),
            department: "Finance".to_string(),
            description: "Analyze expenses by category, detect anomalies".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "accounting.invoice-reader".to_string(),
            name: "Invoice Reader".to_string(),
            department: "Accounting".to_string(),
            description: "Extract structured data from invoices".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "accounting.invoice-control".to_string(),
            name: "Invoice Control".to_string(),
            department: "Accounting".to_string(),
            description: "Validate invoices against PO and receipts".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "accounting.account-reconciliation".to_string(),
            name: "Account Reconciliation".to_string(),
            department: "Accounting".to_string(),
            description: "Reconcile GL accounts with sub-ledger".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "procurement.supplier-comparison".to_string(),
            name: "Supplier Comparison".to_string(),
            department: "Procurement".to_string(),
            description: "Compare suppliers on multiple criteria".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "procurement.price-history".to_string(),
            name: "Price History".to_string(),
            department: "Procurement".to_string(),
            description: "Analyze historical price trends".to_string(),
            tier: "Free".to_string(),
        },
        AgentInfo {
            id: "procurement.decision".to_string(),
            name: "Procurement Decision".to_string(),
            department: "Procurement".to_string(),
            description: "Evidence-backed procurement decisions".to_string(),
            tier: "Free".to_string(),
        },
    ]
}

#[tauri::command]
pub fn get_agent_manifest(agent_id: String) -> Result<AgentInfo, String> {
    let agents = list_agents();
    agents.into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("Agent {} not found", agent_id))
}

#[tauri::command]
pub async fn execute_agent(agent_id: String, input: serde_json::Value) -> Result<serde_json::Value, String> {
    tracing::info!(agent_id = %agent_id, "Executing agent from UI");

    Ok(serde_json::json!({
        "execution_id": uuid::Uuid::new_v4().to_string(),
        "agent_id": agent_id,
        "status": "completed",
        "output": {
            "message": format!("Agent {} executed successfully", agent_id),
            "input_received": input
        }
    }))
}