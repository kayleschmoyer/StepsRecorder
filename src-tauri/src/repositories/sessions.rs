use rusqlite::{params, Connection, OptionalExtension};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    models::{
        AppErrorResponse, RecordingSession, RecordingStatus, SessionDetail, SessionSummary,
        StartRecordingSessionInput, StopRecordingSessionInput, UpdateSessionInput,
    },
    repositories::steps,
};

pub fn start_recording_session(
    connection: &Connection,
    input: StartRecordingSessionInput,
) -> Result<RecordingSession, AppErrorResponse> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        ensure_no_active_recording_session(connection)?;
        let session_id = create_recording_session(connection, input)?;
        mark_session_as_recording(connection, &session_id)?;
        get_recording_session(connection, &session_id)
    })();

    finish_transaction(connection, result)
}

pub fn stop_recording_session(
    connection: &Connection,
    input: StopRecordingSessionInput,
) -> Result<RecordingSession, AppErrorResponse> {
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(to_database_write_error)?;

    let result = (|| {
        let active_session_id = get_active_recording_session_id(connection)?.ok_or_else(|| {
            AppErrorResponse::new(
                "recording_session_not_active",
                "There is no active recording session to stop.",
            )
        })?;

        if active_session_id != input.session_id {
            return Err(AppErrorResponse::new(
                "recording_session_mismatch",
                "The requested session is not the active recording session.",
            ));
        }

        mark_session_as_completed(connection, &input.session_id)?;
        get_recording_session(connection, &input.session_id)
    })();

    finish_transaction(connection, result)
}

pub fn get_recording_status(connection: &Connection) -> Result<RecordingStatus, AppErrorResponse> {
    let active = connection
        .query_row(
            "SELECT id,
                    CAST(MAX(0, ROUND((julianday('now') - julianday(started_at)) * 86400)) AS INTEGER),
                    step_count
             FROM recording_sessions
             WHERE status = 'recording'
             LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(to_database_error)?;

    Ok(match active {
        Some((session_id, elapsed_seconds, step_count)) => RecordingStatus {
            is_recording: true,
            active_session_id: Some(session_id),
            elapsed_seconds: Some(elapsed_seconds),
            step_count,
        },
        None => RecordingStatus {
            is_recording: false,
            active_session_id: None,
            elapsed_seconds: None,
            step_count: 0,
        },
    })
}

pub fn list_sessions(
    connection: &Connection,
    limit: Option<i64>,
    include_archived: bool,
) -> Result<Vec<SessionSummary>, AppErrorResponse> {
    let limit = limit.unwrap_or(10).clamp(1, 100);

    let sql = if include_archived {
        "SELECT id, title, status, started_at, ended_at, step_count
         FROM recording_sessions
         ORDER BY updated_at DESC
         LIMIT ?1"
    } else {
        "SELECT id, title, status, started_at, ended_at, step_count
         FROM recording_sessions
         WHERE status != 'archived'
         ORDER BY updated_at DESC
         LIMIT ?1"
    };

    let mut statement = connection.prepare(sql).map_err(to_database_error)?;
    let sessions = statement
        .query_map(params![limit], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                ended_at: row.get(4)?,
                step_count: row.get(5)?,
            })
        })
        .map_err(to_database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_database_error)?;

    Ok(sessions)
}

pub fn get_session(
    connection: &Connection,
    session_id: &str,
) -> Result<SessionDetail, AppErrorResponse> {
    let session = connection
        .query_row(
            "SELECT id, title, description, status, started_at, ended_at
             FROM recording_sessions
             WHERE id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .optional()
        .map_err(to_database_error)?
        .ok_or_else(|| {
            AppErrorResponse::new("session_not_found", "The requested session was not found.")
        })?;

    let steps = steps::list_active_steps_for_session(connection, session_id)?;

    Ok(SessionDetail {
        id: session.0,
        title: session.1,
        description: session.2,
        status: session.3,
        started_at: session.4,
        ended_at: session.5,
        steps,
    })
}

pub fn update_session(
    connection: &Connection,
    input: UpdateSessionInput,
) -> Result<RecordingSession, AppErrorResponse> {
    let existing = get_recording_session(connection, &input.session_id)?;

    let title = input.title.unwrap_or(existing.title);
    if title.trim().is_empty() {
        return Err(AppErrorResponse::new(
            "session_invalid_title",
            "Session title cannot be empty.",
        ));
    }

    let description = input.description.or(existing.description);
    let include_timestamps_default = input
        .include_timestamps_default
        .unwrap_or(existing.include_timestamps_default);
    let include_click_markers_default = input
        .include_click_markers_default
        .unwrap_or(existing.include_click_markers_default);

    connection
        .execute(
            "UPDATE recording_sessions
             SET title = ?1,
                 description = ?2,
                 include_timestamps_default = ?3,
                 include_click_markers_default = ?4,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?5",
            params![
                title,
                description,
                bool_to_int(include_timestamps_default),
                bool_to_int(include_click_markers_default),
                input.session_id
            ],
        )
        .map_err(to_database_error)?;

    get_recording_session(connection, &input.session_id)
}

fn create_recording_session(
    connection: &Connection,
    input: StartRecordingSessionInput,
) -> Result<String, AppErrorResponse> {
    let title = input
        .title
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Untitled recording".to_string());
    let description = input
        .description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let session_id = generate_session_id();

    connection
        .execute(
            "INSERT INTO recording_sessions (
                id, title, description, status, started_at, ended_at, created_at, updated_at,
                default_export_directory, step_count, include_timestamps_default,
                include_click_markers_default
            )
            VALUES (
                ?1, ?2, ?3, 'draft',
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                NULL,
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                NULL, 0, 1, 1
            )",
            params![session_id, title, description],
        )
        .map_err(to_database_write_error)?;

    Ok(session_id)
}

fn mark_session_as_recording(
    connection: &Connection,
    session_id: &str,
) -> Result<(), AppErrorResponse> {
    let rows_changed = connection
        .execute(
            "UPDATE recording_sessions
             SET status = 'recording',
                 started_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 ended_at = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1 AND status = 'draft'",
            params![session_id],
        )
        .map_err(to_database_write_error)?;

    if rows_changed == 0 {
        return Err(AppErrorResponse::new(
            "session_not_found",
            "The requested recording session was not found.",
        ));
    }

    Ok(())
}

fn mark_session_as_completed(
    connection: &Connection,
    session_id: &str,
) -> Result<(), AppErrorResponse> {
    let rows_changed = connection
        .execute(
            "UPDATE recording_sessions
             SET status = 'completed',
                 ended_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1 AND status = 'recording'",
            params![session_id],
        )
        .map_err(to_database_write_error)?;

    if rows_changed == 0 {
        return Err(AppErrorResponse::new(
            "recording_session_not_active",
            "The requested session is not currently recording.",
        ));
    }

    Ok(())
}

fn ensure_no_active_recording_session(connection: &Connection) -> Result<(), AppErrorResponse> {
    if get_active_recording_session_id(connection)?.is_some() {
        return Err(AppErrorResponse::new(
            "recording_session_already_active",
            "Stop the active recording session before starting another one.",
        ));
    }

    Ok(())
}

fn get_active_recording_session_id(
    connection: &Connection,
) -> Result<Option<String>, AppErrorResponse> {
    connection
        .query_row(
            "SELECT id FROM recording_sessions WHERE status = 'recording' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(to_database_error)
}

fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("recording-session-{nanos}-{}", std::process::id())
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

fn get_recording_session(
    connection: &Connection,
    session_id: &str,
) -> Result<RecordingSession, AppErrorResponse> {
    connection
        .query_row(
            "SELECT id, title, description, status, started_at, ended_at, created_at, updated_at,
                    default_export_directory, step_count, include_timestamps_default,
                    include_click_markers_default
             FROM recording_sessions
             WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(RecordingSession {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    started_at: row.get(4)?,
                    ended_at: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    default_export_directory: row.get(8)?,
                    step_count: row.get(9)?,
                    include_timestamps_default: row.get::<_, i64>(10)? == 1,
                    include_click_markers_default: row.get::<_, i64>(11)? == 1,
                })
            },
        )
        .optional()
        .map_err(to_database_error)?
        .ok_or_else(|| {
            AppErrorResponse::new("session_not_found", "The requested session was not found.")
        })
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn to_database_write_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local recording session data could not be updated.",
        error.to_string(),
    )
}

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("open in-memory database");
        connection
            .execute_batch(
                r#"
                CREATE TABLE recording_sessions (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    status TEXT NOT NULL CHECK (status IN ('draft', 'recording', 'completed', 'exported', 'archived')),
                    started_at TEXT NOT NULL,
                    ended_at TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    default_export_directory TEXT,
                    step_count INTEGER NOT NULL DEFAULT 0 CHECK (step_count >= 0),
                    include_timestamps_default INTEGER NOT NULL DEFAULT 1 CHECK (include_timestamps_default IN (0, 1)),
                    include_click_markers_default INTEGER NOT NULL DEFAULT 1 CHECK (include_click_markers_default IN (0, 1))
                );

                CREATE UNIQUE INDEX idx_recording_sessions_single_active
                    ON recording_sessions(status)
                    WHERE status = 'recording';

                CREATE TABLE recording_steps (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    step_number INTEGER NOT NULL CHECK (step_number > 0),
                    title TEXT NOT NULL,
                    description TEXT,
                    action_type TEXT NOT NULL CHECK (action_type IN ('click')),
                    captured_at TEXT NOT NULL,
                    click_x INTEGER,
                    click_y INTEGER,
                    monitor_id TEXT,
                    app_window_title TEXT,
                    process_name TEXT,
                    original_screenshot_path TEXT NOT NULL,
                    edited_screenshot_path TEXT,
                    thumbnail_path TEXT,
                    is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (session_id) REFERENCES recording_sessions(id) ON DELETE CASCADE
                );

                CREATE UNIQUE INDEX idx_recording_steps_active_step_number
                    ON recording_steps(session_id, step_number)
                    WHERE is_deleted = 0;
                "#,
            )
            .expect("create test schema");
        connection
    }

    #[test]
    fn recording_lifecycle_creates_one_active_session_then_completes_it() {
        let connection = test_connection();

        let session = start_recording_session(
            &connection,
            StartRecordingSessionInput {
                title: Some("Lifecycle smoke test".to_string()),
                description: None,
            },
        )
        .expect("start recording session");

        assert_eq!(session.title, "Lifecycle smoke test");
        assert_eq!(session.status, "recording");
        assert_eq!(session.step_count, 0);
        assert!(session.ended_at.is_none());

        let active_status = get_recording_status(&connection).expect("get active status");
        assert!(active_status.is_recording);
        assert_eq!(
            active_status.active_session_id.as_deref(),
            Some(session.id.as_str())
        );
        assert_eq!(active_status.step_count, 0);
        assert!(active_status.elapsed_seconds.unwrap_or_default() >= 0);

        let duplicate_error = start_recording_session(
            &connection,
            StartRecordingSessionInput {
                title: None,
                description: None,
            },
        )
        .expect_err("second active recording should fail");
        assert_eq!(duplicate_error.code, "recording_session_already_active");

        let completed = stop_recording_session(
            &connection,
            StopRecordingSessionInput {
                session_id: session.id.clone(),
            },
        )
        .expect("stop active recording");

        assert_eq!(completed.id, session.id);
        assert_eq!(completed.status, "completed");
        assert!(completed.ended_at.is_some());
        assert_eq!(completed.step_count, 0);

        let inactive_status = get_recording_status(&connection).expect("get inactive status");
        assert!(!inactive_status.is_recording);
        assert!(inactive_status.active_session_id.is_none());
        assert_eq!(inactive_status.step_count, 0);

        let detail =
            get_session(&connection, &completed.id).expect("open completed zero-step session");
        assert_eq!(detail.status, "completed");
        assert!(detail.steps.is_empty());

        let sessions = list_sessions(&connection, Some(5), false).expect("list recent sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, completed.id);
        assert_eq!(sessions[0].step_count, 0);
    }
}
