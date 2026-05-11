use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const MAX_LOG_ENTRIES: usize = 1000;
const LOG_EVENT: &str = "diagnostic-log";

/// 前端诊断面板使用的结构化日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticLogEntry {
    pub time: String,
    pub level: String,
    pub module: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone)]
pub struct DiagnosticLogger {
    entries: Arc<Mutex<VecDeque<DiagnosticLogEntry>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl DiagnosticLogger {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn attach_app(&self, app_handle: AppHandle) {
        if let Ok(mut handle) = self.app_handle.lock() {
            *handle = Some(app_handle);
        }
    }

    pub fn recent(&self) -> Vec<DiagnosticLogEntry> {
        self.entries
            .lock()
            .map(|entries| entries.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn push(
        &self,
        level: &str,
        module: &str,
        message: impl Into<String>,
        station_id: Option<impl Into<String>>,
        station_name: Option<impl Into<String>>,
        detail: Option<impl Into<String>>,
    ) {
        let entry = DiagnosticLogEntry {
            time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level: level.to_string(),
            module: module.to_string(),
            message: message.into(),
            station_id: station_id.map(Into::into),
            station_name: station_name.map(Into::into),
            detail: detail.map(Into::into),
        };

        if let Ok(mut entries) = self.entries.lock() {
            if entries.len() >= MAX_LOG_ENTRIES {
                entries.pop_front();
            }
            entries.push_back(entry.clone());
        }

        if let Ok(handle) = self.app_handle.lock() {
            if let Some(app) = handle.as_ref() {
                let _ = app.emit(LOG_EVENT, entry);
            }
        }
    }

    pub fn info(&self, module: &str, message: impl Into<String>) {
        self.push(
            "info",
            module,
            message,
            None::<String>,
            None::<String>,
            None::<String>,
        );
    }

    pub fn warn(
        &self,
        module: &str,
        message: impl Into<String>,
        detail: Option<impl Into<String>>,
    ) {
        self.push(
            "warn",
            module,
            message,
            None::<String>,
            None::<String>,
            detail,
        );
    }

    pub fn error(
        &self,
        module: &str,
        message: impl Into<String>,
        detail: Option<impl Into<String>>,
    ) {
        self.push(
            "error",
            module,
            message,
            None::<String>,
            None::<String>,
            detail,
        );
    }
}

impl Default for DiagnosticLogger {
    fn default() -> Self {
        Self::new()
    }
}
