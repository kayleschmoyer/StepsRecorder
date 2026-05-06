use tauri::State;

use crate::{
    db::AppDatabase,
    models::{
        AppErrorResponse, AppSettings, GetSessionInput, ListSessionsInput, RecordingSession,
        SessionDetail, SessionSummary, UpdateSessionInput, UpdateSettingsInput,
    },
    repositories::{sessions, settings},
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
