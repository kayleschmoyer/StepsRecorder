use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSession {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub default_export_directory: Option<String>,
    pub step_count: i64,
    pub include_timestamps_default: bool,
    pub include_click_markers_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub step_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub steps: Vec<RecordingStep>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStep {
    pub id: String,
    pub session_id: String,
    pub step_number: i64,
    pub title: String,
    pub description: Option<String>,
    pub action_type: String,
    pub captured_at: String,
    pub click_x: Option<i64>,
    pub click_y: Option<i64>,
    pub monitor_id: Option<String>,
    pub app_window_title: Option<String>,
    pub process_name: Option<String>,
    pub original_screenshot_path: String,
    pub edited_screenshot_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotEdit {
    pub id: String,
    pub step_id: String,
    pub edit_type: String,
    pub edit_data_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub screenshot_mode: String,
    pub click_debounce_ms: i64,
    pub include_timestamps_in_export: bool,
    pub include_click_markers: bool,
    pub privacy_reminder_before_export: bool,
    pub default_export_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHistoryRecord {
    pub id: String,
    pub session_id: String,
    pub export_type: String,
    pub output_path: String,
    pub exported_at: String,
    pub include_timestamps: bool,
    pub include_click_markers: bool,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListScreenshotEditsInput {
    pub step_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExportHistoryInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSessionsInput {
    pub limit: Option<i64>,
    pub include_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionInput {
    pub session_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub include_timestamps_default: Option<bool>,
    pub include_click_markers_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStepInput {
    pub step_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteStepInput {
    pub step_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteStepResult {
    pub step_id: String,
    pub session_id: String,
    pub deleted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderStepsInput {
    pub session_id: String,
    pub ordered_step_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderStepsResult {
    pub session_id: String,
    pub steps: Vec<RecordingStep>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsInput {
    pub click_debounce_ms: Option<i64>,
    pub include_timestamps_in_export: Option<bool>,
    pub include_click_markers: Option<bool>,
    pub privacy_reminder_before_export: Option<bool>,
    pub default_export_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl AppErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
}
