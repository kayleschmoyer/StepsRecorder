use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    capture::service::CapturedClickEvent,
    models::{
        AppErrorResponse, DeleteStepInput, DeleteStepResult, RecordingStep, ReorderStepsInput,
        ReorderStepsResult, UpdateStepInput,
    },
};

pub const MISSING_SCREENSHOT_PATH: &str = "";
const APP_WINDOW_TITLE_MAX_CHARS: usize = 200;
const PROCESS_NAME_MAX_CHARS: usize = 100;

pub fn create_recorded_click_step(
    connection: &Connection,
    event: &CapturedClickEvent,
) -> Result<RecordingStep, AppErrorResponse> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        ensure_session_is_recording(connection, &event.session_id)?;

        let step_number = next_step_number(connection, &event.session_id)?;
        let step_id = generate_step_id();
        let title = format!("Step {step_number}: Click recorded");
        let safe_process_name =
            sanitize_optional_metadata(event.process_name.as_deref(), PROCESS_NAME_MAX_CHARS);
        let safe_window_title = sanitize_optional_metadata(
            event.active_window_title.as_deref(),
            APP_WINDOW_TITLE_MAX_CHARS,
        );
        let process_label = safe_process_name
            .as_deref()
            .unwrap_or("unknown application");
        let description = format!(
            "Click captured at ({}, {}) in {}.",
            event.x, event.y, process_label
        );
        let captured_at_unix_seconds = event
            .timestamp_ms
            .saturating_div(1_000)
            .min(i64::MAX as u128) as i64;

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
                    strftime('%Y-%m-%dT%H:%M:%fZ', ?6, 'unixepoch'),
                    ?7, ?8, ?9, ?10, ?11, ?12, NULL, NULL, 0,
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                )",
                params![
                    step_id,
                    event.session_id,
                    step_number,
                    title,
                    description,
                    captured_at_unix_seconds,
                    event.x,
                    event.y,
                    event.monitor_id,
                    safe_window_title,
                    safe_process_name,
                    MISSING_SCREENSHOT_PATH
                ],
            )
            .map_err(to_database_write_error)?;

        refresh_session_step_count(connection, &event.session_id)?;
        get_active_step(connection, &step_id)
    })();

    finish_transaction(connection, result)
}

pub fn update_original_screenshot_path(
    connection: &Connection,
    step_id: &str,
    original_screenshot_path: &str,
) -> Result<RecordingStep, AppErrorResponse> {
    if original_screenshot_path.trim().is_empty() {
        return Err(AppErrorResponse::new(
            "screenshot_path_empty",
            "Screenshot path cannot be empty after capture succeeds.",
        ));
    }

    connection
        .execute(
            "UPDATE recording_steps
             SET original_screenshot_path = ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?2 AND is_deleted = 0",
            params![original_screenshot_path.trim(), step_id],
        )
        .map_err(to_database_write_error)?;

    get_active_step(connection, step_id)
}

pub fn update_edited_screenshot_path(
    connection: &Connection,
    step_id: &str,
    edited_screenshot_path: &str,
) -> Result<RecordingStep, AppErrorResponse> {
    if edited_screenshot_path.trim().is_empty() {
        return Err(AppErrorResponse::new(
            "screenshot_path_empty",
            "Edited screenshot path cannot be empty after marker generation succeeds.",
        ));
    }

    connection
        .execute(
            "UPDATE recording_steps
             SET edited_screenshot_path = ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?2 AND is_deleted = 0",
            params![edited_screenshot_path.trim(), step_id],
        )
        .map_err(to_database_write_error)?;

    get_active_step(connection, step_id)
}

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

pub fn update_step(
    connection: &Connection,
    input: UpdateStepInput,
) -> Result<RecordingStep, AppErrorResponse> {
    let existing = get_active_step(connection, &input.step_id)?;
    let title = input.title.unwrap_or(existing.title);

    if title.trim().is_empty() {
        return Err(AppErrorResponse::new(
            "step_invalid_title",
            "Step title cannot be empty.",
        ));
    }

    let description = input.description.or(existing.description);

    connection
        .execute(
            "UPDATE recording_steps
             SET title = ?1,
                 description = ?2,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?3 AND is_deleted = 0",
            params![title.trim(), description, input.step_id],
        )
        .map_err(to_database_write_error)?;

    get_active_step(connection, &input.step_id)
}

pub fn delete_step(
    connection: &Connection,
    input: DeleteStepInput,
) -> Result<DeleteStepResult, AppErrorResponse> {
    let existing = get_active_step(connection, &input.step_id)?;

    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        connection
            .execute(
                "UPDATE recording_steps
                 SET is_deleted = 1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?1 AND is_deleted = 0",
                params![input.step_id],
            )
            .map_err(to_database_write_error)?;

        compact_active_step_numbers(connection, &existing.session_id)?;
        refresh_session_step_count(connection, &existing.session_id)?;
        Ok(DeleteStepResult {
            step_id: existing.id,
            session_id: existing.session_id,
            deleted: true,
        })
    })();

    finish_transaction(connection, result)
}

pub fn reorder_steps(
    connection: &Connection,
    input: ReorderStepsInput,
) -> Result<ReorderStepsResult, AppErrorResponse> {
    let active_steps = list_active_steps_for_session(connection, &input.session_id)?;

    if active_steps.len() != input.ordered_step_ids.len() {
        return Err(AppErrorResponse::new(
            "steps_reorder_mismatch",
            "The reorder request must include every active step exactly once.",
        ));
    }

    let active_ids = active_steps
        .iter()
        .map(|step| step.id.as_str())
        .collect::<HashSet<_>>();
    let requested_ids = input
        .ordered_step_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    if active_ids.len() != requested_ids.len() || active_ids != requested_ids {
        return Err(AppErrorResponse::new(
            "steps_reorder_mismatch",
            "The reorder request can only include active steps from the requested session.",
        ));
    }

    let temporary_base = active_steps
        .iter()
        .map(|step| step.step_number)
        .max()
        .unwrap_or(0)
        + active_steps.len() as i64
        + 1;

    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        for (index, step_id) in input.ordered_step_ids.iter().enumerate() {
            connection
                .execute(
                    "UPDATE recording_steps
                     SET step_number = ?1,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?2 AND session_id = ?3 AND is_deleted = 0",
                    params![
                        temporary_step_number(temporary_base, index),
                        step_id,
                        input.session_id
                    ],
                )
                .map_err(to_database_write_error)?;
        }

        for (index, step_id) in input.ordered_step_ids.iter().enumerate() {
            connection
                .execute(
                    "UPDATE recording_steps
                     SET step_number = ?1,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?2 AND session_id = ?3 AND is_deleted = 0",
                    params![(index as i64) + 1, step_id, input.session_id],
                )
                .map_err(to_database_write_error)?;
        }

        refresh_session_step_count(connection, &input.session_id)?;
        let steps = list_active_steps_for_session(connection, &input.session_id)?;

        Ok(ReorderStepsResult {
            session_id: input.session_id,
            steps,
        })
    })();

    finish_transaction(connection, result)
}

fn ensure_session_is_recording(
    connection: &Connection,
    session_id: &str,
) -> Result<(), AppErrorResponse> {
    let status = connection
        .query_row(
            "SELECT status FROM recording_sessions WHERE id = ?1",
            params![session_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(to_database_error)?
        .ok_or_else(|| {
            AppErrorResponse::new(
                "session_not_found",
                "The active recording session was not found.",
            )
        })?;

    if status != "recording" {
        return Err(AppErrorResponse::new(
            "recording_session_not_active",
            "The click was accepted after the recording session stopped.",
        ));
    }

    Ok(())
}

fn next_step_number(connection: &Connection, session_id: &str) -> Result<i64, AppErrorResponse> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(step_number), 0) + 1
             FROM recording_steps
             WHERE session_id = ?1 AND is_deleted = 0",
            params![session_id],
            |row| row.get(0),
        )
        .map_err(to_database_error)
}

fn generate_step_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("recording-step-{nanos}-{}", std::process::id())
}

fn sanitize_optional_metadata(value: Option<&str>, max_chars: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_metadata(value, max_chars))
}

fn truncate_metadata(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() && max_chars > 1 {
        let keep = max_chars.saturating_sub(1);
        let mut readable: String = truncated.chars().take(keep).collect();
        readable.push('…');
        readable
    } else {
        truncated
    }
}

pub fn get_active_step(
    connection: &Connection,
    step_id: &str,
) -> Result<RecordingStep, AppErrorResponse> {
    connection
        .query_row(
            "SELECT id, session_id, step_number, title, description, action_type, captured_at,
                    click_x, click_y, monitor_id, app_window_title, process_name,
                    original_screenshot_path, edited_screenshot_path, thumbnail_path,
                    is_deleted, created_at, updated_at
             FROM recording_steps
             WHERE id = ?1 AND is_deleted = 0",
            params![step_id],
            map_recording_step,
        )
        .optional()
        .map_err(to_database_error)?
        .ok_or_else(|| {
            AppErrorResponse::new("step_not_found", "The requested active step was not found.")
        })
}

fn compact_active_step_numbers(
    connection: &Connection,
    session_id: &str,
) -> Result<(), AppErrorResponse> {
    let active_steps = list_active_steps_for_session(connection, session_id)?;
    let temporary_base = active_steps
        .iter()
        .map(|step| step.step_number)
        .max()
        .unwrap_or(0)
        + active_steps.len() as i64
        + 1;

    for (index, step) in active_steps.iter().enumerate() {
        connection
            .execute(
                "UPDATE recording_steps
                 SET step_number = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?2 AND session_id = ?3 AND is_deleted = 0",
                params![
                    temporary_step_number(temporary_base, index),
                    step.id,
                    session_id
                ],
            )
            .map_err(to_database_write_error)?;
    }

    for (index, step) in active_steps.iter().enumerate() {
        connection
            .execute(
                "UPDATE recording_steps
                 SET step_number = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?2 AND session_id = ?3 AND is_deleted = 0",
                params![(index as i64) + 1, step.id, session_id],
            )
            .map_err(to_database_write_error)?;
    }

    Ok(())
}

fn refresh_session_step_count(
    connection: &Connection,
    session_id: &str,
) -> Result<(), AppErrorResponse> {
    connection
        .execute(
            "UPDATE recording_sessions
             SET step_count = (
                 SELECT COUNT(*)
                 FROM recording_steps
                 WHERE session_id = ?1 AND is_deleted = 0
             ),
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
            params![session_id],
        )
        .map_err(to_database_write_error)?;

    Ok(())
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

fn temporary_step_number(base: i64, index: usize) -> i64 {
    base + index as i64
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

fn to_database_write_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be updated.",
        error.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_truncation_keeps_values_readable() {
        let long_title = "A".repeat(APP_WINDOW_TITLE_MAX_CHARS + 50);
        let long_process_name = "process".repeat(30);

        let title = sanitize_optional_metadata(Some(&long_title), APP_WINDOW_TITLE_MAX_CHARS)
            .expect("truncated title");
        let process_name =
            sanitize_optional_metadata(Some(&long_process_name), PROCESS_NAME_MAX_CHARS)
                .expect("truncated process name");

        assert_eq!(title.chars().count(), APP_WINDOW_TITLE_MAX_CHARS);
        assert!(title.ends_with('…'));
        assert_eq!(process_name.chars().count(), PROCESS_NAME_MAX_CHARS);
        assert!(process_name.ends_with('…'));
    }
}
