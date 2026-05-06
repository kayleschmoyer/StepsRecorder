use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    models::{
        AppErrorResponse, RecordingSession, SessionDetail, SessionSummary, UpdateSessionInput,
    },
    repositories::steps,
};

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

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}
