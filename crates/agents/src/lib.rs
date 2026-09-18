pub mod accounting;
pub mod csv_util;
pub mod finance;
pub mod hr;
pub mod legal;
pub mod logistics;
pub mod management;
pub mod operations;
pub mod pmo;
pub mod procurement;
pub mod registry;
pub mod sales;

pub use registry::{all_agents, all_manifests, register_all};

pub use accounting::{
    AccountReconciliationAgent, InvoiceControlAgent, InvoiceReaderAgent, JournalEntryAgent,
    ReconcileReportAgent, TaxComplianceAgent,
};
pub use finance::{
    BankReconciliationAgent, BudgetVarianceAgent, CashFlowForecastAgent, ExpenseAnalystAgent,
    FinancialRiskAgent, RevenueRecognitionAgent,
};
pub use hr::{
    AttritionRiskAgent, CompensationAnalyzerAgent, EmployeeEngagementAgent, HeadcountPlannerAgent,
    HiringPipelineAgent, TrainingROIAgent,
};
pub use legal::{
    ComplianceCheckAgent, ContractAnalyzerAgent, DeadlineTrackerAgent, DocumentReviewerAgent,
    LegalRiskAssessmentAgent, RegulatoryMonitorAgent,
};
pub use logistics::{
    DeliveryTrackerAgent, FleetManagerAgent, FreightAnalyzerAgent, LastMileAgent,
    RouteOptimizerAgent, WarehouseOptimizerAgent,
};
pub use management::{
    BoardReportAgent, BudgetTrackerAgent, DecisionMatrixAgent, KPIReporterAgent, OKRAgent,
    StrategicInitiativeAgent,
};
pub use operations::{
    CapacityPlannerAgent, InventoryOptimizerAgent, MaintenancePlannerAgent,
    ProductionSchedulerAgent, QualityAssuranceAgent, SupplyChainRiskAgent,
};
pub use pmo::{
    ChangeRequestAgent, LessonsLearnedAgent, ProjectTrackerAgent, ResourceAllocatorAgent,
    RiskRegisterAgent, StakeholderReportAgent,
};
pub use procurement::{
    PriceHistoryAgent, ProcurementDecisionAgent, PurchaseOrderValidatorAgent, SpendAnalyticsAgent,
    SupplierComparisonAgent, VendorRiskAgent,
};
pub use sales::{
    CustomerSegmentationAgent, LeadScoringAgent, PipelineHealthAgent, PricingOptimizationAgent,
    SalesForecastAgent, WinLossAnalysisAgent,
};
