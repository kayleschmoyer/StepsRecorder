use rusqlite::{params, Connection};

use crate::models::{AppErrorResponse, ExportHistoryRecord};

pub fn list_export_history_for_session(
    connection: &Connection,
    session_id: &str,
) -> Result<Vec<ExportHistoryRecord>, AppErrorResponse> {
    let mut statement = connection
        .prepare(
            "SELECT id, session_id, export_type, output_path, exported_at,
                    include_timestamps, include_click_markers, status, error_message
             FROM export_history
             WHERE session_id = ?1
             ORDER BY exported_at DESC",
        )
        .map_err(to_database_error)?;

    let records = statement
        .query_map(params![session_id], |row| {
            Ok(ExportHistoryRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                export_type: row.get(2)?,
                output_path: row.get(3)?,
                exported_at: row.get(4)?,
                include_timestamps: row.get::<_, i64>(5)? == 1,
                include_click_markers: row.get::<_, i64>(6)? == 1,
                status: row.get(7)?,
                error_message: row.get(8)?,
            })
        })
        .map_err(to_database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_database_error)?;

    Ok(records)
}

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}
