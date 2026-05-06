# Architecture

## Architecture Goal

The architecture must keep the app clean, modular, testable, and safe.

This is a desktop app with two major layers:

1. Frontend UI layer using React, TypeScript, and Vite.
2. Native system layer using Tauri 2 and Rust.

The frontend should provide the user experience. The Rust layer should handle Windows-level functionality.

## Recommended Stack

### Desktop Runtime

- Tauri 2
- Rust
- Windows API access through Rust crates and/or direct Windows bindings

### Frontend

- React 19
- TypeScript
- Vite
- CSS Modules or vanilla CSS using design tokens
- Zustand for UI state

### Data

- SQLite for local structured app data
- File system for screenshots and exports

### Testing

- Vitest
- React Testing Library
- Playwright
- Rust unit/integration tests

## High-Level Data Flow

```text
User clicks Start
  -> React calls Tauri command
  -> Rust starts global mouse capture
  -> User clicks in target app
  -> Rust receives click event
  -> Rust captures screenshot of clicked monitor
  -> Rust stores screenshot file
  -> Rust writes step metadata to SQLite
  -> React receives recording update
  -> User stops recording
  -> React loads session steps
  -> User edits steps
  -> React saves edits through Tauri command
  -> User exports document
  -> Rust generates DOCX or PDF
```

## Folder Structure

Recommended structure:

```text
/
  README.md
  package.json
  vite.config.ts
  tsconfig.json

  /docs
    AI_AGENT_RULES.md
    PRODUCT_REQUIREMENTS.md
    ARCHITECTURE.md
    DESIGN_SYSTEM.md
    CODING_STANDARDS.md
    DATABASE_SCHEMA.md
    API_CONTRACT.md
    TESTING_STRATEGY.md
    SECURITY_REQUIREMENTS.md
    ERROR_HANDLING.md
    PERFORMANCE_REQUIREMENTS.md
    ACCESSIBILITY.md
    DEPLOYMENT.md
    ROADMAP.md

  /src
    /app
      App.tsx
      routes.tsx

    /assets

    /components
      /button
      /card
      /dialog
      /empty-state
      /form
      /layout
      /toast
      /toolbar

    /features
      /recording
        RecordingHomePage.tsx
        RecordingControlBar.tsx
        RecordingStatusPanel.tsx
        recordingStore.ts
        recordingTypes.ts

      /sessions
        SessionReviewPage.tsx
        StepList.tsx
        StepCard.tsx
        StepEditorPanel.tsx
        sessionsStore.ts
        sessionsTypes.ts

      /screenshot-editor
        ScreenshotEditorPage.tsx
        CropTool.tsx
        RedactionTool.tsx
        AnnotationToolbar.tsx
        screenshotEditorStore.ts

      /export
        ExportDialog.tsx
        ExportPreview.tsx
        exportTypes.ts

      /settings
        SettingsPage.tsx
        settingsStore.ts

    /lib
      tauriClient.ts
      dateFormat.ts
      fileSize.ts

    /styles
      tokens.css
      global.css

    /types
      app.ts
      session.ts
      step.ts
      export.ts

  /src-tauri
    /src
      main.rs
      lib.rs

      /commands
        recording_commands.rs
        session_commands.rs
        screenshot_commands.rs
        export_commands.rs
        settings_commands.rs

      /capture
        mouse_hook.rs
        screen_capture.rs
        monitor_detection.rs
        click_debounce.rs

      /database
        connection.rs
        migrations.rs
        session_repository.rs
        step_repository.rs
        settings_repository.rs

      /export
        docx_exporter.rs
        pdf_exporter.rs
        document_model.rs

      /files
        app_paths.rs
        screenshot_store.rs
        export_store.rs
        cleanup.rs

      /models
        session.rs
        step.rs
        settings.rs
        export.rs

      /errors
        app_error.rs
        error_codes.rs
```

## Responsibility Boundaries

### React Frontend Responsibilities

The frontend handles:

- Navigation
- Layout
- Forms
- Buttons
- Modals
- Toasts
- Review/edit UI
- Screenshot editing UI
- Export flow UI
- Settings UI
- Calling Tauri commands
- Displaying errors

The frontend must not:

- Capture mouse input globally
- Capture screenshots directly
- Write directly to SQLite
- Generate final DOCX/PDF files
- Access system APIs directly

### Rust/Tauri Backend Responsibilities

The Rust/Tauri layer handles:

- Global mouse click detection
- Screenshot capture
- Monitor detection
- File system operations
- SQLite reads/writes
- DOCX generation
- PDF generation
- App path management
- Cleanup of orphaned files
- Native error handling

## Tauri Command Pattern

Frontend code calls typed wrapper functions in `src/lib/tauriClient.ts`.

Do not call `invoke()` directly throughout the app.

Preferred pattern:

```typescript
// UI component
await recordingApi.startRecordingSession(input);

// tauriClient.ts
export const recordingApi = {
  startRecordingSession: (input: StartRecordingInput) =>
    invoke<RecordingSession>("start_recording_session", { input }),
};
```

## State Management

Use Zustand for UI state that spans multiple components.

Examples:

- Current recording status
- Selected session
- Selected step
- Screenshot editor state
- Toast notifications
- Export dialog state

Do not store large images directly in Zustand.

## Local Data Rules

SQLite stores metadata only.

Screenshots are stored as image files.

Exports are stored as `.docx` or `.pdf` files.

## Screenshot Storage

Recommended app data structure:

```text
%APPDATA%/StepForgeRecorder/
  /data
    stepforge.sqlite

  /screenshots
    /session-{sessionId}
      step-0001-original.png
      step-0001-edited.png

  /exports
    /session-{sessionId}
      workflow-guide.docx
      workflow-guide.pdf

  /logs
    app.log
```

## Capture Mode

MVP capture mode:

```text
Capture the visible monitor where the click occurred.
```

This matches the user's requirement that the screenshot reflect what was visible on the screen, not hidden windows behind it.

## Export Architecture

Exports must use a structured document model.

```text
Session
  -> DocumentModel
  -> DOCX Exporter
  -> PDF Exporter
```

Do not generate exports by scraping rendered HTML from the UI unless explicitly approved.

## Error Handling Architecture

All Rust errors should map to known app error codes.

All frontend errors should display friendly messages.

Detailed technical errors should be logged locally without exposing sensitive data in the UI.

## Modularity Rules

- One component should have one clear purpose.
- One service should have one clear responsibility.
- Avoid utility dumping grounds.
- Avoid deeply nested prop chains.
- Avoid global mutable state outside defined stores.
- Prefer typed models shared conceptually between frontend and backend.

## Future Architecture Considerations

Future versions may add:

- OCR
- AI-generated step descriptions
- Active-window capture mode
- Target-app-only capture
- Team workspace sync
- Jira/Xray export
- OneDrive/SharePoint export
- Redaction detection
