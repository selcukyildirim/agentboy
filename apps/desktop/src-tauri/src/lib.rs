mod app_state;
mod commands;
mod cost;
mod llm_factory;
mod provider_bridge;
mod resilience;

pub use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_app_version,
            commands::list_providers,
            commands::configure_provider,
            commands::test_provider,
            commands::get_provider_status,
            commands::remove_provider_credential,
            commands::list_provider_models,
            commands::set_active_provider,
            commands::get_active_provider,
            commands::list_agents,
            commands::get_agent_manifest,
            commands::execute_agent,
            commands::list_workflows,
            commands::create_workflow,
            commands::add_workflow_step,
            commands::delete_workflow,
            commands::execute_workflow,
            commands::list_documents,
            commands::upload_document,
            commands::query_rag,
            commands::get_cache_stats,
            commands::clear_cache,
            commands::get_health,
            commands::get_metrics,
            commands::list_decision_types,
            commands::get_decision_context,
            commands::get_recommendation,
            commands::list_executions,
            commands::get_execution,
            commands::get_execution_steps,
            commands::delete_execution,
            commands::get_execution_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
