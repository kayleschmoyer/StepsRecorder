use rusqlite::{params, Connection};

use crate::models::{AppErrorResponse, ClearSeededDataResult, SessionDetail};
use crate::repositories::sessions;

const DEV_SEED_SESSION_ID: &str = "dev-seed-session-settings-review";

const DEV_SEED_STEPS: [(&str, i64, &str, &str, &str, i64, i64, &str, &str); 3] = [
    (
        "dev-seed-step-001-open-settings",
        1,
        "Open Settings from the home screen",
        "Use the Settings shortcut to review capture preferences before starting a recording.",
        "dev-only-placeholder://screenshots/open-settings",
        214,
        118,
        "Steps Recorder — Home",
        "steps-recorder.exe",
    ),
    (
        "dev-seed-step-002-adjust-debounce",
        2,
        "Adjust click debounce timing",
        "Update the debounce value so repeated clicks are filtered during future native capture testing.",
        "dev-only-placeholder://screenshots/adjust-debounce",
        486,
        362,
        "Steps Recorder — Settings",
        "steps-recorder.exe",
    ),
    (
        "dev-seed-step-003-review-session",
        3,
        "Review captured workflow steps",
        "Open Session Review to edit step text, move steps up or down, and delete mistakes before export exists.",
        "dev-only-placeholder://screenshots/review-session",
        742,
        510,
        "Steps Recorder — Session Review",
        "steps-recorder.exe",
    ),
];

/// Development-only fixture used before native mouse/screenshot capture exists.
///
/// This inserts placeholder screenshot path strings only and intentionally does not
/// create image files or screenshot edit records.
pub fn seed_sample_data(connection: &Connection) -> Result<SessionDetail, AppErrorResponse> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        clear_seeded_data_inner(connection)?;

        connection
            .execute(
                "INSERT INTO recording_sessions (
                    id, title, description, status, started_at, ended_at, created_at, updated_at,
                    default_export_directory, step_count, include_timestamps_default,
                    include_click_markers_default
                )
                VALUES (
                    ?1, ?2, ?3, 'completed',
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-12 minutes'),
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-4 minutes'),
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-12 minutes'),
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                    NULL, 3, 1, 1
                )",
                params![
                    DEV_SEED_SESSION_ID,
                    "DEV ONLY — Settings and Session Review sample",
                    "Development-only fixture for testing settings persistence and Session Review step edit, delete, and reorder commands before native capture is implemented."
                ],
            )
            .map_err(to_database_write_error)?;

        for (
            step_id,
            step_number,
            title,
            description,
            placeholder_path,
            click_x,
            click_y,
            window_title,
            process_name,
        ) in DEV_SEED_STEPS
        {
            connection
                .execute(
                    "INSERT INTO recording_steps (
                        id, session_id, step_number, title, description, action_type, captured_at,
                        click_x, click_y, monitor_id, app_window_title, process_name,
                        original_screenshot_path, edited_screenshot_path, thumbnail_path,
                        is_deleted, created_at, updated_at
                    )
                    VALUES (
                        ?1, ?2, ?3, ?4, ?5, 'click',
                        strftime('%Y-%m-%dT%H:%M:%fZ', 'now', printf('-%d minutes', 12 - ?3)),
                        ?6, ?7, 'primary-monitor', ?8, ?9, ?10, NULL, NULL, 0,
                        strftime('%Y-%m-%dT%H:%M:%fZ', 'now', printf('-%d minutes', 12 - ?3)),
                        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    )",
                    params![
                        step_id,
                        DEV_SEED_SESSION_ID,
                        step_number,
                        title,
                        description,
                        click_x,
                        click_y,
                        window_title,
                        process_name,
                        placeholder_path
                    ],
                )
                .map_err(to_database_write_error)?;
        }

        sessions::get_session(connection, DEV_SEED_SESSION_ID)
    })();

    finish_transaction(connection, result)
}

/// Development-only cleanup for removing only the deterministic seeded fixture.
pub fn clear_seeded_data(
    connection: &Connection,
) -> Result<ClearSeededDataResult, AppErrorResponse> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = clear_seeded_data_inner(connection);
    finish_transaction(connection, result)
}

fn clear_seeded_data_inner(
    connection: &Connection,
) -> Result<ClearSeededDataResult, AppErrorResponse> {
    let deleted_steps = connection
        .execute(
            "DELETE FROM recording_steps
             WHERE session_id = ?1 OR id LIKE 'dev-seed-step-%'",
            params![DEV_SEED_SESSION_ID],
        )
        .map_err(to_database_write_error)?;

    let deleted_sessions = connection
        .execute(
            "DELETE FROM recording_sessions
             WHERE id = ?1",
            params![DEV_SEED_SESSION_ID],
        )
        .map_err(to_database_write_error)?;

    Ok(ClearSeededDataResult {
        session_id: DEV_SEED_SESSION_ID.to_string(),
        deleted_sessions,
        deleted_steps,
    })
}

fn finish_transaction<T>(
    connection: &Connection,
    result: Result<T, AppErrorResponse>,
) -> Result<T, AppErrorResponse> {
    match result {
        Ok(value) => {
            connection
                .execute_batch("COMMIT")
                .map_err(to_database_write_error)?;
            Ok(value)
        }
        Err(error) => {
            let _ = connection.execute_batch("ROLLBACK");
            Err(error)
        }
    }
}

fn to_database_write_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local development fixture data could not be updated.",
        error.to_string(),
    )
}
