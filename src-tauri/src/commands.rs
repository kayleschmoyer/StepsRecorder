use tauri::State;

use crate::{
    capture::CaptureService,
    db::AppDatabase,
    models::{
        AppErrorResponse, AppSettings, ClearSeededDataResult, DeleteStepInput, DeleteStepResult,
        ExportHistoryRecord, GetSessionInput, ListExportHistoryInput, ListScreenshotEditsInput,
        ListSessionsInput, RecordingSession, RecordingStatus, RecordingStep, ReorderStepsInput,
        ReorderStepsResult, ScreenshotEdit, SessionDetail, SessionSummary,
        StartRecordingSessionInput, StopRecordingSessionInput, UpdateSessionInput,
        UpdateSettingsInput, UpdateStepInput,
    },
    repositories::{export_history, screenshot_edits, sessions, settings, steps},
};

#[tauri::command]
pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn get_settings(database: State<'_, AppDatabase>) -> Result<AppSettings, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    settings::get_settings(&connection)
}

#[tauri::command]
pub fn update_settings(
    input: UpdateSettingsInput,
    database: State<'_, AppDatabase>,
) -> Result<AppSettings, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    settings::update_settings(&connection, input)
}

#[tauri::command]
pub fn start_recording_session(
    input: StartRecordingSessionInput,
    database: State<'_, AppDatabase>,
    capture_service: State<'_, CaptureService>,
) -> Result<RecordingSession, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    let app_settings = settings::get_settings(&connection)?;
    let session = sessions::start_recording_session(&connection, input)?;
    drop(connection);

    capture_service.start(session.id.clone(), app_settings.click_debounce_ms)?;

    Ok(session)
}

#[tauri::command]
pub fn stop_recording_session(
    input: StopRecordingSessionInput,
    database: State<'_, AppDatabase>,
    capture_service: State<'_, CaptureService>,
) -> Result<RecordingSession, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    let session = sessions::stop_recording_session(&connection, input)?;
    drop(connection);

    capture_service.stop(&session.id)?;

    Ok(session)
}

#[tauri::command]
pub fn get_recording_status(
    database: State<'_, AppDatabase>,
) -> Result<RecordingStatus, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    sessions::get_recording_status(&connection)
}

#[tauri::command]
pub fn list_sessions(
    input: Option<ListSessionsInput>,
    database: State<'_, AppDatabase>,
) -> Result<Vec<SessionSummary>, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;
    let input = input.unwrap_or(ListSessionsInput {
        limit: None,
        include_archived: None,
    });

    sessions::list_sessions(
        &connection,
        input.limit,
        input.include_archived.unwrap_or(false),
    )
}

#[tauri::command]
pub fn get_session(
    input: GetSessionInput,
    database: State<'_, AppDatabase>,
) -> Result<SessionDetail, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    sessions::get_session(&connection, &input.session_id)
}

#[tauri::command]
pub fn update_session(
    input: UpdateSessionInput,
    database: State<'_, AppDatabase>,
) -> Result<RecordingSession, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    sessions::update_session(&connection, input)
}

#[tauri::command]
pub fn list_screenshot_edits(
    input: ListScreenshotEditsInput,
    database: State<'_, AppDatabase>,
) -> Result<Vec<ScreenshotEdit>, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    screenshot_edits::list_screenshot_edits_for_step(&connection, &input.step_id)
}

#[tauri::command]
pub fn list_export_history(
    input: ListExportHistoryInput,
    database: State<'_, AppDatabase>,
) -> Result<Vec<ExportHistoryRecord>, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    export_history::list_export_history_for_session(&connection, &input.session_id)
}

#[tauri::command]
pub fn update_step(
    input: UpdateStepInput,
    database: State<'_, AppDatabase>,
) -> Result<RecordingStep, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    steps::update_step(&connection, input)
}

#[tauri::command]
pub fn delete_step(
    input: DeleteStepInput,
    database: State<'_, AppDatabase>,
) -> Result<DeleteStepResult, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    steps::delete_step(&connection, input)
}

#[tauri::command]
pub fn reorder_steps(
    input: ReorderStepsInput,
    database: State<'_, AppDatabase>,
) -> Result<ReorderStepsResult, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    steps::reorder_steps(&connection, input)
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn dev_seed_sample_data(
    database: State<'_, AppDatabase>,
) -> Result<SessionDetail, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    // Development-only fixture command: inserts deterministic sample metadata
    // with placeholder screenshot path strings only. It is registered only in
    // debug builds and must not be presented as a production feature.
    crate::repositories::dev_fixtures::seed_sample_data(&connection)
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn dev_clear_seeded_data(
    database: State<'_, AppDatabase>,
) -> Result<ClearSeededDataResult, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    // Development-only fixture cleanup command: removes only deterministic
    // dev-seed rows and is registered only in debug builds.
    crate::repositories::dev_fixtures::clear_seeded_data(&connection)
}
