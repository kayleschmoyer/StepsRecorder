# API Contract

## Purpose

This file defines the contract between the React frontend and the Rust/Tauri backend.

The frontend must call Tauri commands through typed wrappers. Components should not call `invoke()` directly.

## General Response Pattern

Tauri commands should return typed success values or structured errors.

### Error Shape

```typescript
export interface AppErrorResponse {
  code: string;
  message: string;
  details?: string;
}
```

## Recording Commands

## start_recording_session

Starts a new recording session.

### Input

```typescript
export interface StartRecordingSessionInput {
  title?: string;
  description?: string;
}
```

### Output

```typescript
export interface RecordingSession {
  id: string;
  title: string;
  description?: string;
  status: "draft" | "recording" | "completed" | "exported" | "archived";
  startedAt: string;
  endedAt?: string;
  stepCount: number;
}
```

### Rules

- Creates a session if one does not already exist.
- Starts global click capture.
- Returns the active recording session.
- Fails if another session is already recording.

## stop_recording_session

Stops the active recording session.

### Input

```typescript
export interface StopRecordingSessionInput {
  sessionId: string;
}
```

### Output

```typescript
export interface RecordingSession {
  id: string;
  title: string;
  status: "completed";
  startedAt: string;
  endedAt: string;
  stepCount: number;
}
```

### Rules

- Stops global click capture.
- Sets `endedAt`.
- Marks session as completed.
- Does not delete captured steps.

## get_recording_status

Gets current recording status.

### Input

None.

### Output

```typescript
export interface RecordingStatus {
  isRecording: boolean;
  activeSessionId?: string;
  elapsedSeconds?: number;
  stepCount: number;
}
```

## Session Commands

## list_sessions

Returns recent sessions.

### Input

```typescript
export interface ListSessionsInput {
  limit?: number;
  includeArchived?: boolean;
}
```

### Output

```typescript
export interface SessionSummary {
  id: string;
  title: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  stepCount: number;
}
```

## get_session

Gets one session and its steps.

### Input

```typescript
export interface GetSessionInput {
  sessionId: string;
}
```

### Output

```typescript
export interface SessionDetail {
  id: string;
  title: string;
  description?: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  steps: RecordingStep[];
}
```

## update_session

Updates session title/description/default export options.

### Input

```typescript
export interface UpdateSessionInput {
  sessionId: string;
  title?: string;
  description?: string;
  includeTimestampsDefault?: boolean;
  includeClickMarkersDefault?: boolean;
}
```

### Output

```typescript
export interface RecordingSession {
  id: string;
  title: string;
  description?: string;
  status: string;
  stepCount: number;
}
```

## Step Commands

## update_step

Updates step title/description.

### Input

```typescript
export interface UpdateStepInput {
  stepId: string;
  title?: string;
  description?: string;
}
```

### Output

```typescript
export interface RecordingStep {
  id: string;
  sessionId: string;
  stepNumber: number;
  title: string;
  description?: string;
  originalScreenshotPath: string;
  editedScreenshotPath?: string;
}
```

## delete_step

Marks a step as deleted.

### Input

```typescript
export interface DeleteStepInput {
  stepId: string;
}
```

### Output

```typescript
export interface DeleteStepResult {
  stepId: string;
  sessionId: string;
  deleted: boolean;
}
```

## reorder_steps

Reorders active steps in a session.

### Input

```typescript
export interface ReorderStepsInput {
  sessionId: string;
  orderedStepIds: string[];
}
```

### Output

```typescript
export interface ReorderStepsResult {
  sessionId: string;
  steps: RecordingStep[];
}
```

## Screenshot Commands

## get_step_screenshot_preview

Returns a base64 PNG preview for Session Review. The command prefers an existing marked/edited screenshot and falls back to the immutable original screenshot.

### Input

```typescript
export interface GetStepScreenshotPreviewInput {
  stepId: string;
}
```

### Output

```typescript
export interface StepScreenshotPreview {
  exists: boolean;
  originalScreenshotPath: string;
  editedScreenshotPath?: string;
  displayedScreenshotPath?: string;
  previewKind: "missing" | "original" | "click_marker";
  dataUrl?: string;
}
```

## save_screenshot_edit

Saves an edited screenshot copy.

### Input

```typescript
export interface SaveScreenshotEditInput {
  stepId: string;
  editedImagePath: string;
  editType: "crop" | "redact" | "highlight" | "arrow" | "text" | "combined";
}
```

### Output

```typescript
export interface SaveScreenshotEditResult {
  stepId: string;
  editedScreenshotPath: string;
}
```

## Export Commands

## export_session

Exports a session as DOCX or PDF.

### Input

```typescript
export interface ExportSessionInput {
  sessionId: string;
  exportType: "docx" | "pdf";
  outputPath: string;
  includeTimestamps: boolean;
  includeClickMarkers: boolean;
  includeTitlePage: boolean;
}
```

### Output

```typescript
export interface ExportSessionResult {
  sessionId: string;
  exportType: "docx" | "pdf";
  outputPath: string;
  exportedAt: string;
}
```

### Rules

- Fails if session has no active steps.
- Uses edited screenshots when available.
- Uses original screenshots otherwise.
- Writes an export history record.
- Returns a friendly error if export fails.

## Settings Commands

## get_settings

Returns app settings.

### Input

None.

### Output

```typescript
export interface AppSettings {
  screenshotMode: "clicked_monitor" | "clicked_window";
  clickDebounceMs: number;
  includeTimestampsInExport: boolean;
  includeClickMarkers: boolean;
  privacyReminderBeforeExport: boolean;
  defaultExportDirectory?: string;
}
```

## update_settings

Updates app settings.

### Input

```typescript
export interface UpdateSettingsInput {
  screenshotMode?: "clicked_monitor" | "clicked_window";
  clickDebounceMs?: number;
  includeTimestampsInExport?: boolean;
  includeClickMarkers?: boolean;
  privacyReminderBeforeExport?: boolean;
  defaultExportDirectory?: string;
}
```

### Output

```typescript
export interface AppSettings {
  screenshotMode: "clicked_monitor" | "clicked_window";
  clickDebounceMs: number;
  includeTimestampsInExport: boolean;
  includeClickMarkers: boolean;
  privacyReminderBeforeExport: boolean;
  defaultExportDirectory?: string;
}
```

## Event Payloads

The Rust backend may emit events to the frontend.

## recording_step_captured

Emitted when a new step is captured.

```typescript
export interface RecordingStepCapturedEvent {
  sessionId: string;
  stepId: string;
  stepNumber: number;
  capturedAt: string;
}
```

## recording_error

Emitted when a non-fatal recording error occurs.

```typescript
export interface RecordingErrorEvent {
  sessionId?: string;
  code: string;
  message: string;
}
```

## Contract Rules

- Do not add commands without updating this file.
- Do not change input/output shapes without updating this file.
- Components should use typed wrappers, not raw `invoke()` calls.
- All command errors must be user-safe.
