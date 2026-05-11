//! 实时诊断日志相关命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::diagnostics::DiagnosticLogEntry;
use crate::AppState;

/// 获取最近诊断日志。
#[tauri::command]
pub async fn get_diagnostic_logs(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<DiagnosticLogEntry>, String> {
    let state = state.lock().await;
    Ok(state.logger.recent())
}

/// 清空诊断日志。
#[tauri::command]
pub async fn clear_diagnostic_logs(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.logger.clear();
    state.logger.info("diagnostics", "诊断日志已清空");
    Ok(())
}
