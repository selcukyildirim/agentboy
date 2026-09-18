pub mod accounting;
pub mod csv_util;
pub mod finance;
pub mod hr;
pub mod logistics;
pub mod management;
pub mod operations;
pub mod pmo;
pub mod procurement;
pub mod registry;
pub mod sales;
pub mod legal;

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
    AttritionRiskAgent, HeadcountPlannerAgent, CompensationAnalyzerAgent,
    TrainingROIAgent, EmployeeEngagementAgent, HiringPipelineAgent,
};
pub use logistics::{
    RouteOptimizerAgent, FleetManagerAgent, WarehouseOptimizerAgent,
    DeliveryTrackerAgent, FreightAnalyzerAgent, LastMileAgent,
};
pub use management::{
    KPIReporterAgent, StrategicInitiativeAgent, BudgetTrackerAgent,
    BoardReportAgent, OKRAgent, DecisionMatrixAgent,
};
pub use operations::{
    InventoryOptimizerAgent, ProductionSchedulerAgent, QualityAssuranceAgent,
    CapacityPlannerAgent, MaintenancePlannerAgent, SupplyChainRiskAgent,
};
pub use pmo::{
    ProjectTrackerAgent, RiskRegisterAgent, ResourceAllocatorAgent,
    StakeholderReportAgent, ChangeRequestAgent, LessonsLearnedAgent,
};
pub use procurement::{
    PriceHistoryAgent, ProcurementDecisionAgent, SupplierComparisonAgent,
    PurchaseOrderValidatorAgent, SpendAnalyticsAgent, VendorRiskAgent,
};
pub use sales::{
    SalesForecastAgent, LeadScoringAgent, PipelineHealthAgent,
    WinLossAnalysisAgent, CustomerSegmentationAgent, PricingOptimizationAgent,
};
pub use legal::{
    ContractAnalyzerAgent, ComplianceCheckAgent, LegalRiskAssessmentAgent,
    DocumentReviewerAgent, DeadlineTrackerAgent, RegulatoryMonitorAgent,
};
