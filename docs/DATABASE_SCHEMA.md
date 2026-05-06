# Database Schema

## Storage Strategy

The app uses SQLite for local structured data.

Screenshots and exports are stored as files on disk.

The database stores file paths and metadata, not image blobs.

## Database File

Recommended path:

```text
%APPDATA%/StepForgeRecorder/data/stepforge.sqlite
```

## Tables

## recording_sessions

Stores one recording session.

### Fields

| Field | Type | Required | Notes |
|---|---:|---:|---|
| id | TEXT | Yes | UUID |
| title | TEXT | Yes | User-editable |
| description | TEXT | No | Optional session notes |
| status | TEXT | Yes | `draft`, `recording`, `completed`, `exported`, `archived` |
| started_at | TEXT | Yes | ISO timestamp |
| ended_at | TEXT | No | ISO timestamp |
| created_at | TEXT | Yes | ISO timestamp |
| updated_at | TEXT | Yes | ISO timestamp |
| default_export_directory | TEXT | No | Optional |
| step_count | INTEGER | Yes | Cached count |
| include_timestamps_default | INTEGER | Yes | 0 or 1 |
| include_click_markers_default | INTEGER | Yes | 0 or 1 |

### Rules

- `id` must be unique.
- `status` must be a known value.
- `step_count` must match active, non-deleted steps where possible.
- `ended_at` is null while recording.

## recording_steps

Stores one recorded step.

### Fields

| Field | Type | Required | Notes |
|---|---:|---:|---|
| id | TEXT | Yes | UUID |
| session_id | TEXT | Yes | FK to recording_sessions.id |
| step_number | INTEGER | Yes | 1-based display order |
| title | TEXT | Yes | User-editable |
| description | TEXT | No | User-editable |
| action_type | TEXT | Yes | `click` for MVP |
| captured_at | TEXT | Yes | ISO timestamp |
| click_x | INTEGER | No | Screen coordinate |
| click_y | INTEGER | No | Screen coordinate |
| monitor_id | TEXT | No | Display identifier |
| app_window_title | TEXT | No | Best-effort active window title |
| process_name | TEXT | No | Best-effort active process name |
| original_screenshot_path | TEXT | Yes | File path; during Step 8 metadata-only click persistence this is the documented placeholder `SCREENSHOT_CAPTURE_PENDING_STEP_8_METADATA_ONLY` and no image file is written. |
| edited_screenshot_path | TEXT | No | File path for a derived non-destructive image, including the Step 10 generated click-marker preview (`step-0001-marked.png`); original files remain in `original_screenshot_path`. |
| thumbnail_path | TEXT | No | Optional preview image |
| is_deleted | INTEGER | Yes | 0 or 1 |
| created_at | TEXT | Yes | ISO timestamp |
| updated_at | TEXT | Yes | ISO timestamp |

### Rules

- A step must belong to a session.
- A step must have a screenshot path. Step 8 metadata-only clicks use the placeholder `SCREENSHOT_CAPTURE_PENDING_STEP_8_METADATA_ONLY` until screenshot capture is implemented; this placeholder is not a real file path.
- `step_number` must be unique within active steps for the session.
- Session Review and future export flows use `edited_screenshot_path` when present; otherwise they use `original_screenshot_path`. Step 10 stores generated click-marker previews in `edited_screenshot_path` without overwriting originals.
- Deleted steps are excluded from export.

## screenshot_edits

Stores non-destructive screenshot edit metadata where feasible.

### Fields

| Field | Type | Required | Notes |
|---|---:|---:|---|
| id | TEXT | Yes | UUID |
| step_id | TEXT | Yes | FK to recording_steps.id |
| edit_type | TEXT | Yes | `crop`, `redact`, `highlight`, `arrow`, `text` |
| edit_data_json | TEXT | Yes | JSON payload for edit geometry/settings |
| created_at | TEXT | Yes | ISO timestamp |

### Rules

- `edit_data_json` is allowed here because it stores flexible editor geometry, not app configuration.
- The user's general preference is to avoid JSON config files, but SQLite JSON payloads for complex edit metadata are acceptable if kept isolated and documented.
- Final rendered screenshot should be saved as an edited image file.

## app_settings

Stores local user settings.

### Fields

| Field | Type | Required | Notes |
|---|---:|---:|---|
| key | TEXT | Yes | Unique setting key |
| value | TEXT | Yes | Setting value |
| updated_at | TEXT | Yes | ISO timestamp |

### Required Settings

| Key | Default | Notes |
|---|---|---|
| screenshot_mode | `clicked_monitor` | MVP default |
| click_debounce_ms | `500` | Prevents accidental rapid duplicates |
| include_timestamps_in_export | `true` | Default export option |
| include_click_markers | `true` | Default screenshot marker option |
| privacy_reminder_before_export | `true` | User safety |
| default_export_directory | empty | User-selected path |

## export_history

Stores generated export records.

### Fields

| Field | Type | Required | Notes |
|---|---:|---:|---|
| id | TEXT | Yes | UUID |
| session_id | TEXT | Yes | FK to recording_sessions.id |
| export_type | TEXT | Yes | `docx` or `pdf` |
| output_path | TEXT | Yes | Local path |
| exported_at | TEXT | Yes | ISO timestamp |
| include_timestamps | INTEGER | Yes | 0 or 1 |
| include_click_markers | INTEGER | Yes | 0 or 1 |
| status | TEXT | Yes | `success`, `failed` |
| error_message | TEXT | No | User-safe message if failed |

## Schema Change Rules

- Do not rename fields without migration notes.
- Do not remove fields without migration notes.
- Any new table must be documented here before implementation.
- Any new field must be documented here before implementation.
- Migrations must be deterministic.
- Migrations must not delete user screenshots or exports.
