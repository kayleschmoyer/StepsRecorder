use base64::{engine::general_purpose, Engine as _};
use tauri::State;

use crate::{
    capture::{screenshot::ScreenshotStorage, CaptureService},
    db::AppDatabase,
    models::{
        AppErrorResponse, AppSettings, ClearSeededDataResult, DeleteStepInput, DeleteStepResult,
        ExportHistoryRecord, GetSessionInput, GetStepScreenshotPreviewInput,
        ListExportHistoryInput, ListScreenshotEditsInput, ListSessionsInput, RecordingSession,
        RecordingStatus, RecordingStep, ReorderStepsInput, ReorderStepsResult,
        SaveEditedScreenshotInput, ScreenshotEdit, SessionDetail, SessionSummary,
        StartRecordingSessionInput, StepScreenshotPreview, StopRecordingSessionInput,
        UpdateSessionInput, UpdateSettingsInput, UpdateStepInput,
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
pub fn get_step_screenshot_preview(
    input: GetStepScreenshotPreviewInput,
    database: State<'_, AppDatabase>,
) -> Result<StepScreenshotPreview, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    let step = steps::get_active_step(&connection, &input.step_id)?;
    drop(connection);

    if step.original_screenshot_path.trim().is_empty() {
        return Ok(StepScreenshotPreview {
            exists: false,
            original_screenshot_path: step.original_screenshot_path,
            edited_screenshot_path: step.edited_screenshot_path,
            displayed_screenshot_path: None,
            preview_kind: "missing".to_string(),
            data_url: None,
        });
    }

    let preferred_path = step
        .edited_screenshot_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty());
    let (preview_path, preview_kind) = match preferred_path {
        Some(path) if std::path::Path::new(path).exists() => {
            let kind = if path.ends_with("-edited.png") {
                "edited"
            } else {
                "click_marker"
            };
            (path.to_string(), kind)
        }
        Some(path) => {
            eprintln!(
                "screenshot.preview event=marked_missing step_id={} edited_path={}",
                step.id, path
            );
            (step.original_screenshot_path.clone(), "original")
        }
        None => (step.original_screenshot_path.clone(), "original"),
    };

    let bytes = match std::fs::read(&preview_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StepScreenshotPreview {
                exists: false,
                original_screenshot_path: step.original_screenshot_path,
                edited_screenshot_path: step.edited_screenshot_path,
                displayed_screenshot_path: None,
                preview_kind: "missing".to_string(),
                data_url: None,
            });
        }
        Err(error) => {
            return Err(AppErrorResponse::with_details(
                "screenshot_preview_read_error",
                "The screenshot preview file could not be read.",
                error.to_string(),
            ));
        }
    };

    let encoded = general_purpose::STANDARD.encode(bytes);
    Ok(StepScreenshotPreview {
        exists: true,
        original_screenshot_path: step.original_screenshot_path,
        edited_screenshot_path: step.edited_screenshot_path,
        displayed_screenshot_path: Some(preview_path),
        preview_kind: preview_kind.to_string(),
        data_url: Some(format!("data:image/png;base64,{encoded}")),
    })
}

#[tauri::command]
pub fn save_edited_screenshot(
    input: SaveEditedScreenshotInput,
    database: State<'_, AppDatabase>,
    screenshot_storage: State<'_, ScreenshotStorage>,
) -> Result<RecordingStep, AppErrorResponse> {
    let connection = database.connection.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "database_lock_error",
            "The local app database is currently unavailable.",
            error.to_string(),
        )
    })?;

    let step = steps::get_active_step(&connection, &input.step_id)?;
    let original_path = step.original_screenshot_path.trim();
    if original_path.is_empty() || !std::path::Path::new(original_path).exists() {
        return Err(AppErrorResponse::new(
            "screenshot_source_missing",
            "The original screenshot must exist before an edited screenshot can be saved.",
        ));
    }

    let edited_path = screenshot_storage.edited_path_for_step(&step.session_id, step.step_number);
    write_data_url_png_atomically(&edited_path, &input.screenshot_data_url)?;
    let edited_path = edited_path.to_string_lossy().to_string();
    steps::update_edited_screenshot_path(&connection, &step.id, &edited_path)
}

fn write_data_url_png_atomically(
    output_path: &std::path::Path,
    screenshot_data_url: &str,
) -> Result<(), AppErrorResponse> {
    let trimmed = screenshot_data_url.trim();
    let encoded = trimmed
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(|| {
            AppErrorResponse::new(
                "edited_screenshot_invalid_data",
                "Edited screenshot data must be a PNG data URL.",
            )
        })?;
    let bytes = general_purpose::STANDARD.decode(encoded).map_err(|error| {
        AppErrorResponse::with_details(
            "edited_screenshot_decode_error",
            "The edited screenshot data could not be decoded.",
            error.to_string(),
        )
    })?;

    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(AppErrorResponse::new(
            "edited_screenshot_invalid_png",
            "Edited screenshots must be saved as PNG images.",
        ));
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppErrorResponse::with_details(
                "edited_screenshot_directory_error",
                "The edited screenshot directory could not be created.",
                error.to_string(),
            )
        })?;
    }

    let temp_path = output_path.with_extension("png.tmp");
    let backup_path = output_path.with_extension("png.bak");
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&backup_path);

    std::fs::write(&temp_path, bytes).map_err(|error| {
        AppErrorResponse::with_details(
            "edited_screenshot_write_error",
            "The edited screenshot file could not be staged.",
            error.to_string(),
        )
    })?;

    let had_existing = output_path.exists();
    if had_existing {
        std::fs::rename(output_path, &backup_path).map_err(|error| {
            let _ = std::fs::remove_file(&temp_path);
            AppErrorResponse::with_details(
                "edited_screenshot_replace_error",
                "The existing edited screenshot could not be prepared for replacement.",
                error.to_string(),
            )
        })?;
    }

    if let Err(error) = std::fs::rename(&temp_path, output_path) {
        if had_existing {
            let _ = std::fs::rename(&backup_path, output_path);
        }
        let _ = std::fs::remove_file(&temp_path);
        return Err(AppErrorResponse::with_details(
            "edited_screenshot_replace_error",
            "The edited screenshot file could not be saved.",
            error.to_string(),
        ));
    }

    let _ = std::fs::remove_file(&backup_path);
    Ok(())
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
