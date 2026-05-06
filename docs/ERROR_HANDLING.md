# Error Handling

## Goal

Errors must be clear, friendly, and useful.

The user should understand what went wrong and what to do next without seeing raw technical failures.

## Error Philosophy

Bad:

```text
Unhandled exception: HRESULT 0x80070005
```

Good:

```text
Screenshot capture failed because Windows denied access. Try restarting the app as administrator or check screen capture permissions.
```

## Error Categories

Use stable error categories:

- `RECORDING_ERROR`
- `CAPTURE_ERROR`
- `SCREENSHOT_ERROR`
- `DATABASE_ERROR`
- `FILE_SYSTEM_ERROR`
- `EXPORT_ERROR`
- `SETTINGS_ERROR`
- `VALIDATION_ERROR`
- `UNKNOWN_ERROR`

## User-Facing Error Shape

```typescript
export interface UserFacingError {
  code: string;
  title: string;
  message: string;
  recoveryAction?: string;
}
```

## Frontend Error Rules

Frontend must:

- Show friendly error messages.
- Avoid raw stack traces.
- Provide next actions where possible.
- Keep the user in control.
- Avoid losing recorded data when possible.

## Backend Error Rules

Rust backend must:

- Return structured errors.
- Log technical details safely.
- Avoid panics for recoverable errors.
- Avoid exposing sensitive paths/content unless needed.
- Include stable error codes.

## Common Error Examples

## Recording Already Active

Title:

```text
Recording already in progress
```

Message:

```text
Stop the current recording before starting a new one.
```

## Screenshot Capture Failed

Title:

```text
Screenshot could not be captured
```

Message:

```text
The app detected your click, but Windows did not return a screenshot.
```

Recovery:

```text
Try again, or restart the app if this continues.
```

## Export Failed: Missing Screenshot

Title:

```text
Export could not be created
```

Message:

```text
One or more screenshots are missing from this recording.
```

Recovery:

```text
Review the affected steps or remove them before exporting.
```

## File Permission Error

Title:

```text
File could not be saved
```

Message:

```text
The selected folder may not allow files to be created there.
```

Recovery:

```text
Choose a different export folder.
```

## Database Error

Title:

```text
Recording data could not be saved
```

Message:

```text
The app had trouble saving your recording data locally.
```

Recovery:

```text
Restart the app. If this continues, check that your user profile has available disk space.
```

## Error Logging

Log entries should include:

- Timestamp
- Error code
- Operation
- Safe technical message
- App version where available

Do not log:

- Screenshot data
- Sensitive user-entered text
- Passwords
- Keystrokes
- Clipboard data

## Toast vs Dialog

Use a toast for:

- Non-blocking errors
- Recoverable minor issues
- Save confirmations

Use a dialog for:

- Export failures
- Data loss risk
- Recording could not start
- Destructive confirmation

## Recovery First

Whenever possible, offer a direct recovery action:

- Retry
- Choose another folder
- Remove missing step
- Open settings
- Restart recording
