use rusqlite::{params, Connection};

use crate::models::{AppErrorResponse, RecordingStep};

pub fn list_active_steps_for_session(
    connection: &Connection,
    session_id: &str,
) -> Result<Vec<RecordingStep>, AppErrorResponse> {
    let mut statement = connection
        .prepare(
            "SELECT id, session_id, step_number, title, description, action_type, captured_at,
                    click_x, click_y, monitor_id, app_window_title, process_name,
                    original_screenshot_path, edited_screenshot_path, thumbnail_path,
                    is_deleted, created_at, updated_at
             FROM recording_steps
             WHERE session_id = ?1 AND is_deleted = 0
             ORDER BY step_number ASC",
        )
        .map_err(to_database_error)?;

    let steps = statement
        .query_map(params![session_id], map_recording_step)
        .map_err(to_database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_database_error)?;

    Ok(steps)
}

fn map_recording_step(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecordingStep> {
    Ok(RecordingStep {
        id: row.get(0)?,
        session_id: row.get(1)?,
        step_number: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        action_type: row.get(5)?,
        captured_at: row.get(6)?,
        click_x: row.get(7)?,
        click_y: row.get(8)?,
        monitor_id: row.get(9)?,
        app_window_title: row.get(10)?,
        process_name: row.get(11)?,
        original_screenshot_path: row.get(12)?,
        edited_screenshot_path: row.get(13)?,
        thumbnail_path: row.get(14)?,
        is_deleted: row.get::<_, i64>(15)? == 1,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}
