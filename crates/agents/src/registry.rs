use agent_runtime::agent::Agent;
use agent_runtime::manifest::AgentManifest;
use agent_runtime::registry::AgentRegistry;

use crate::accounting::{
    AccountReconciliationAgent, InvoiceControlAgent, InvoiceReaderAgent, JournalEntryAgent,
    ReconcileReportAgent, TaxComplianceAgent,
};
use crate::finance::{
    BankReconciliationAgent, BudgetVarianceAgent, CashFlowForecastAgent, ExpenseAnalystAgent,
    FinancialRiskAgent, RevenueRecognitionAgent,
};
use crate::hr::{
    AttritionRiskAgent, CompensationAnalyzerAgent, EmployeeEngagementAgent, HeadcountPlannerAgent,
    HiringPipelineAgent, TrainingROIAgent,
};
use crate::legal::{
    ComplianceCheckAgent, ContractAnalyzerAgent, DeadlineTrackerAgent, DocumentReviewerAgent,
    LegalRiskAssessmentAgent, RegulatoryMonitorAgent,
};
use crate::logistics::{
    DeliveryTrackerAgent, FleetManagerAgent, FreightAnalyzerAgent, LastMileAgent,
    RouteOptimizerAgent, WarehouseOptimizerAgent,
};
use crate::management::{
    BoardReportAgent, BudgetTrackerAgent, DecisionMatrixAgent, KPIReporterAgent, OKRAgent,
    StrategicInitiativeAgent,
};
use crate::operations::{
    CapacityPlannerAgent, InventoryOptimizerAgent, MaintenancePlannerAgent,
    ProductionSchedulerAgent, QualityAssuranceAgent, SupplyChainRiskAgent,
};
use crate::pmo::{
    ChangeRequestAgent, LessonsLearnedAgent, ProjectTrackerAgent, ResourceAllocatorAgent,
    RiskRegisterAgent, StakeholderReportAgent,
};
use crate::procurement::{
    PriceHistoryAgent, ProcurementDecisionAgent, PurchaseOrderValidatorAgent, SpendAnalyticsAgent,
    SupplierComparisonAgent, VendorRiskAgent,
};
use crate::sales::{
    CustomerSegmentationAgent, LeadScoringAgent, PipelineHealthAgent, PricingOptimizationAgent,
    SalesForecastAgent, WinLossAnalysisAgent,
};

/// All 60 free agents as boxed trait objects.
#[must_use]
pub fn all_agents() -> Vec<Box<dyn Agent>> {
    vec![
        // Finance
        Box::new(BankReconciliationAgent::new()),
        Box::new(BudgetVarianceAgent::new()),
        Box::new(ExpenseAnalystAgent::new()),
        Box::new(FinancialRiskAgent::new()),
        Box::new(RevenueRecognitionAgent::new()),
        Box::new(CashFlowForecastAgent::new()),
        // Accounting
        Box::new(AccountReconciliationAgent::new()),
        Box::new(InvoiceControlAgent::new()),
        Box::new(InvoiceReaderAgent::new()),
        Box::new(TaxComplianceAgent::new()),
        Box::new(JournalEntryAgent::new()),
        Box::new(ReconcileReportAgent::new()),
        // Procurement
        Box::new(PriceHistoryAgent::new()),
        Box::new(ProcurementDecisionAgent::new()),
        Box::new(SupplierComparisonAgent::new()),
        Box::new(PurchaseOrderValidatorAgent::new()),
        Box::new(SpendAnalyticsAgent::new()),
        Box::new(VendorRiskAgent::new()),
        // Sales
        Box::new(SalesForecastAgent::new()),
        Box::new(LeadScoringAgent::new()),
        Box::new(PipelineHealthAgent::new()),
        Box::new(WinLossAnalysisAgent::new()),
        Box::new(CustomerSegmentationAgent::new()),
        Box::new(PricingOptimizationAgent::new()),
        // HR
        Box::new(AttritionRiskAgent::new()),
        Box::new(HeadcountPlannerAgent::new()),
        Box::new(CompensationAnalyzerAgent::new()),
        Box::new(TrainingROIAgent::new()),
        Box::new(EmployeeEngagementAgent::new()),
        Box::new(HiringPipelineAgent::new()),
        // Operations
        Box::new(InventoryOptimizerAgent::new()),
        Box::new(ProductionSchedulerAgent::new()),
        Box::new(QualityAssuranceAgent::new()),
        Box::new(CapacityPlannerAgent::new()),
        Box::new(MaintenancePlannerAgent::new()),
        Box::new(SupplyChainRiskAgent::new()),
        // Logistics
        Box::new(RouteOptimizerAgent::new()),
        Box::new(FleetManagerAgent::new()),
        Box::new(WarehouseOptimizerAgent::new()),
        Box::new(DeliveryTrackerAgent::new()),
        Box::new(FreightAnalyzerAgent::new()),
        Box::new(LastMileAgent::new()),
        // Management
        Box::new(KPIReporterAgent::new()),
        Box::new(StrategicInitiativeAgent::new()),
        Box::new(BudgetTrackerAgent::new()),
        Box::new(BoardReportAgent::new()),
        Box::new(OKRAgent::new()),
        Box::new(DecisionMatrixAgent::new()),
        // PMO
        Box::new(ProjectTrackerAgent::new()),
        Box::new(RiskRegisterAgent::new()),
        Box::new(ResourceAllocatorAgent::new()),
        Box::new(StakeholderReportAgent::new()),
        Box::new(ChangeRequestAgent::new()),
        Box::new(LessonsLearnedAgent::new()),
        // Legal
        Box::new(ContractAnalyzerAgent::new()),
        Box::new(ComplianceCheckAgent::new()),
        Box::new(LegalRiskAssessmentAgent::new()),
        Box::new(DocumentReviewerAgent::new()),
        Box::new(DeadlineTrackerAgent::new()),
        Box::new(RegulatoryMonitorAgent::new()),
    ]
}

/// Register all 60 free agents into a registry.
pub fn register_all(registry: &mut AgentRegistry) {
    for agent in all_agents() {
        registry.register(agent);
    }
}

/// Manifests of all free agents.
#[must_use]
pub fn all_manifests() -> Vec<AgentManifest> {
    all_agents().iter().map(|a| a.manifest()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::manifest::AgentTier;

    #[test]
    fn test_exactly_60_agents() {
        assert_eq!(all_agents().len(), 60);
    }

    #[test]
    fn test_unique_ids() {
        let manifests = all_manifests();
        let mut ids: Vec<&str> = manifests.iter().map(|m| m.id.as_str()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate agent ids found");
    }

    #[test]
    fn test_all_free_tier() {
        for m in all_manifests() {
            assert_eq!(m.tier, AgentTier::Free, "agent {} must be Free", m.id);
        }
    }

    #[test]
    fn test_every_agent_has_input_schema() {
        for m in all_manifests() {
            assert!(
                !m.input_schema.is_empty(),
                "agent {} must declare input_schema",
                m.id
            );
        }
    }

    #[test]
    fn test_input_schema_keys_unique_and_nonempty() {
        for m in all_manifests() {
            let mut keys: Vec<&str> = m.input_schema.iter().map(|f| f.key.as_str()).collect();
            keys.sort_unstable();
            let before = keys.len();
            keys.dedup();
            assert_eq!(before, keys.len(), "duplicate input key in {}", m.id);
            for f in &m.input_schema {
                assert!(!f.key.is_empty());
                assert!(!f.label.is_empty());
            }
        }
    }

    #[test]
    fn test_agents_cover_ten_departments() {
        let mut deps: Vec<String> = all_manifests()
            .iter()
            .map(|m| m.department.clone())
            .collect();
        deps.sort();
        deps.dedup();
        assert_eq!(deps.len(), 10, "expected 10 departments, got {deps:?}");
    }
}
