use agent_runtime::registry::AgentRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;

use agents::{
    AccountReconciliationAgent, InvoiceControlAgent, InvoiceReaderAgent, JournalEntryAgent,
    ReconcileReportAgent, TaxComplianceAgent,
};
use agents::{
    BankReconciliationAgent, BudgetVarianceAgent, CashFlowForecastAgent, ExpenseAnalystAgent,
    FinancialRiskAgent, RevenueRecognitionAgent,
};
use agents::{
    AttritionRiskAgent, HeadcountPlannerAgent, CompensationAnalyzerAgent, TrainingROIAgent,
    EmployeeEngagementAgent, HiringPipelineAgent,
};
use agents::{
    RouteOptimizerAgent, FleetManagerAgent, WarehouseOptimizerAgent, DeliveryTrackerAgent,
    FreightAnalyzerAgent, LastMileAgent,
};
use agents::{
    KPIReporterAgent, StrategicInitiativeAgent, BudgetTrackerAgent, BoardReportAgent, OKRAgent,
    DecisionMatrixAgent,
};
use agents::{
    InventoryOptimizerAgent, ProductionSchedulerAgent, QualityAssuranceAgent,
    CapacityPlannerAgent, MaintenancePlannerAgent, SupplyChainRiskAgent,
};
use agents::{
    ProjectTrackerAgent, RiskRegisterAgent, ResourceAllocatorAgent, StakeholderReportAgent,
    ChangeRequestAgent, LessonsLearnedAgent,
};
use agents::{
    PriceHistoryAgent, ProcurementDecisionAgent, SupplierComparisonAgent,
    PurchaseOrderValidatorAgent, SpendAnalyticsAgent, VendorRiskAgent,
};
use agents::{
    SalesForecastAgent, LeadScoringAgent, PipelineHealthAgent, WinLossAnalysisAgent,
    CustomerSegmentationAgent, PricingOptimizationAgent,
};
use agents::{
    ContractAnalyzerAgent, ComplianceCheckAgent, LegalRiskAssessmentAgent, DocumentReviewerAgent,
    DeadlineTrackerAgent, RegulatoryMonitorAgent,
};

pub struct AppState {
    pub registry: Arc<RwLock<AgentRegistry>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();

        // Finance
        registry.register(Box::new(BankReconciliationAgent::new()));
        registry.register(Box::new(BudgetVarianceAgent::new()));
        registry.register(Box::new(ExpenseAnalystAgent::new()));
        registry.register(Box::new(FinancialRiskAgent::new()));
        registry.register(Box::new(RevenueRecognitionAgent::new()));
        registry.register(Box::new(CashFlowForecastAgent::new()));

        // Accounting
        registry.register(Box::new(AccountReconciliationAgent::new()));
        registry.register(Box::new(InvoiceControlAgent::new()));
        registry.register(Box::new(InvoiceReaderAgent::new()));
        registry.register(Box::new(TaxComplianceAgent::new()));
        registry.register(Box::new(JournalEntryAgent::new()));
        registry.register(Box::new(ReconcileReportAgent::new()));

        // Procurement
        registry.register(Box::new(PriceHistoryAgent::new()));
        registry.register(Box::new(ProcurementDecisionAgent::new()));
        registry.register(Box::new(SupplierComparisonAgent::new()));
        registry.register(Box::new(PurchaseOrderValidatorAgent::new()));
        registry.register(Box::new(SpendAnalyticsAgent::new()));
        registry.register(Box::new(VendorRiskAgent::new()));

        // Sales
        registry.register(Box::new(SalesForecastAgent::new()));
        registry.register(Box::new(LeadScoringAgent::new()));
        registry.register(Box::new(PipelineHealthAgent::new()));
        registry.register(Box::new(WinLossAnalysisAgent::new()));
        registry.register(Box::new(CustomerSegmentationAgent::new()));
        registry.register(Box::new(PricingOptimizationAgent::new()));

        // HR
        registry.register(Box::new(AttritionRiskAgent::new()));
        registry.register(Box::new(HeadcountPlannerAgent::new()));
        registry.register(Box::new(CompensationAnalyzerAgent::new()));
        registry.register(Box::new(TrainingROIAgent::new()));
        registry.register(Box::new(EmployeeEngagementAgent::new()));
        registry.register(Box::new(HiringPipelineAgent::new()));

        // Operations
        registry.register(Box::new(InventoryOptimizerAgent::new()));
        registry.register(Box::new(ProductionSchedulerAgent::new()));
        registry.register(Box::new(QualityAssuranceAgent::new()));
        registry.register(Box::new(CapacityPlannerAgent::new()));
        registry.register(Box::new(MaintenancePlannerAgent::new()));
        registry.register(Box::new(SupplyChainRiskAgent::new()));

        // Logistics
        registry.register(Box::new(RouteOptimizerAgent::new()));
        registry.register(Box::new(FleetManagerAgent::new()));
        registry.register(Box::new(WarehouseOptimizerAgent::new()));
        registry.register(Box::new(DeliveryTrackerAgent::new()));
        registry.register(Box::new(FreightAnalyzerAgent::new()));
        registry.register(Box::new(LastMileAgent::new()));

        // Management
        registry.register(Box::new(KPIReporterAgent::new()));
        registry.register(Box::new(StrategicInitiativeAgent::new()));
        registry.register(Box::new(BudgetTrackerAgent::new()));
        registry.register(Box::new(BoardReportAgent::new()));
        registry.register(Box::new(OKRAgent::new()));
        registry.register(Box::new(DecisionMatrixAgent::new()));

        // PMO
        registry.register(Box::new(ProjectTrackerAgent::new()));
        registry.register(Box::new(RiskRegisterAgent::new()));
        registry.register(Box::new(ResourceAllocatorAgent::new()));
        registry.register(Box::new(StakeholderReportAgent::new()));
        registry.register(Box::new(ChangeRequestAgent::new()));
        registry.register(Box::new(LessonsLearnedAgent::new()));

        // Legal
        registry.register(Box::new(ContractAnalyzerAgent::new()));
        registry.register(Box::new(ComplianceCheckAgent::new()));
        registry.register(Box::new(LegalRiskAssessmentAgent::new()));
        registry.register(Box::new(DocumentReviewerAgent::new()));
        registry.register(Box::new(DeadlineTrackerAgent::new()));
        registry.register(Box::new(RegulatoryMonitorAgent::new()));

        tracing::info!(count = registry.list().len(), "Registered agents");

        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }
}
