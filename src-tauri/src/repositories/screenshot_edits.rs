use rusqlite::{params, Connection};

use crate::models::{AppErrorResponse, ScreenshotEdit};

pub fn list_screenshot_edits_for_step(
    connection: &Connection,
    step_id: &str,
) -> Result<Vec<ScreenshotEdit>, AppErrorResponse> {
    let mut statement = connection
        .prepare(
            "SELECT id, step_id, edit_type, edit_data_json, created_at
             FROM screenshot_edits
             WHERE step_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(to_database_error)?;

    let edits = statement
        .query_map(params![step_id], |row| {
            Ok(ScreenshotEdit {
                id: row.get(0)?,
                step_id: row.get(1)?,
                edit_type: row.get(2)?,
                edit_data_json: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(to_database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_database_error)?;

    Ok(edits)
}

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}
