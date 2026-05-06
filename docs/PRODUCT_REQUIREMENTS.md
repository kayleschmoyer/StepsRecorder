# Product Requirements

## Product Name

Working name: **StepForge Recorder**

The name can change later. The product requirements should remain valid regardless of branding.

## App Goal

Create a modern Windows desktop steps recorder that automatically documents user workflows by recording click-based actions, capturing screenshots, allowing post-recording editing, and exporting polished Microsoft Word or PDF documents.

## Problem Statement

Teams often need to document software workflows, bugs, QA steps, training instructions, support procedures, and implementation processes.

Existing approaches are slow and inconsistent:

- Manual screenshots are tedious.
- Users forget steps.
- Documents look unprofessional.
- QA evidence is scattered.
- Legacy steps recorders feel outdated.
- Editing after capture is painful.

This app should make workflow documentation fast, clean, and reliable.

## MVP Scope

The MVP must support:

1. Start a recording session.
2. Stop a recording session.
3. Detect user mouse clicks while recording.
4. Create a step for each meaningful click.
5. Capture a screenshot of the visible monitor where the click occurred.
6. Store step metadata locally.
7. Display recorded steps in a review screen.
8. Allow the user to edit step title/description.
9. Allow the user to delete steps.
10. Allow the user to reorder steps.
11. Allow basic screenshot editing.
12. Generate a Microsoft Word document.
13. Generate a PDF document.
14. Save generated exports locally.
15. Show friendly errors when recording/export fails.
16. Provide a simple, polished UI.

## Screenshot Editing MVP

The MVP should include at least:

- Redaction rectangle
- Crop
- Save edited screenshot copy

Preferred if feasible:

- Draw highlight rectangle
- Add arrow
- Add text label
- Undo last edit

## Out of Scope for MVP

The MVP will not include:

- Cloud sync
- Team accounts
- Login/authentication
- Browser extension
- Mobile app
- Mac support
- Linux support
- Automatic AI-written descriptions
- OCR-based text recognition
- Video recording
- Keystroke recording
- Audio recording
- Live collaboration
- Direct Jira/Xray integration
- Direct email sending
- Direct Microsoft OneDrive upload
- Direct SharePoint upload

## User Roles

MVP has one local user only.

No role-based access control is required for MVP.

Future roles may include:

- Recorder
- Reviewer
- Admin
- Team Owner

## Core Workflow: Record Steps

### Trigger

User clicks **Start Recording**.

### Flow

1. App shows a recording status indicator.
2. App minimizes or moves into a compact recording control mode.
3. User performs workflow in a Windows desktop application.
4. User clicks somewhere on the screen.
5. App detects the click.
6. App determines the monitor/display where the click occurred.
7. App captures a screenshot of that visible monitor.
8. App creates a new step record.
9. App stores screenshot file locally.
10. App continues recording until user clicks **Stop Recording**.

### Result

A recording session exists with ordered steps and screenshots.

## Core Workflow: Review Steps

### Trigger

User stops recording.

### Flow

1. App opens the session review screen.
2. User sees steps in order.
3. Each step shows:
   - Step number
   - Generated default title
   - Editable description
   - Screenshot preview
   - Timestamp
4. User can edit text.
5. User can delete unnecessary steps.
6. User can reorder steps.
7. User can open a screenshot editor.

### Result

The session is ready for export.

## Core Workflow: Export Document

### Trigger

User clicks **Export**.

### Flow

1. User chooses export type:
   - Microsoft Word `.docx`
   - PDF `.pdf`
2. User chooses save location.
3. App validates session has at least one step.
4. App generates document from structured session data.
5. App saves file locally.
6. App shows success message with file location.

### Result

User has a professional document containing all steps.

## Business Rules

### Recording

- A recording session must have a unique ID.
- A recording session must have a start time.
- A completed recording session must have an end time.
- A step must belong to one session.
- A step must have an order number.
- A step must have a screenshot path.
- A step must have a timestamp.
- Duplicate rapid clicks within a short debounce window should not create excessive unwanted steps.

### Screenshots

- Screenshots are stored as image files on disk.
- The database stores screenshot paths and metadata.
- The original screenshot should be preserved unless the user explicitly deletes it.
- Edited screenshots should be saved separately from originals where possible.
- Export should use the edited screenshot if one exists.

### Editing

- Users can edit step title and description.
- Users can delete steps.
- Users can reorder steps.
- Reordering must update step order numbers.
- Deleted steps should be removed from the final export.
- Deleted screenshot files should not remain orphaned long term.

### Export

- Export should fail gracefully if screenshot files are missing.
- Export should warn the user if there are no steps.
- Export should include the session title.
- Export should include all non-deleted steps in order.
- Export should use consistent document styling.

## Default Step Title Format

When a click is recorded, the app should create a default title.

```text
Step 1: Click recorded
```

Future versions may generate smarter titles, but this is not required for MVP.

## Required Settings

MVP settings:

- Screenshot mode:
  - Monitor where click occurred
- Click debounce interval:
  - Default: 500ms
- Export folder:
  - User selectable
- Include timestamps in export:
  - On/off
- Include click markers on screenshots:
  - On/off
- Privacy reminder before export:
  - On/off

## Privacy Requirements

The app must provide visible warnings that screenshots may capture sensitive information.

Before export, the user should be reminded to review screenshots.

The app must not intentionally record:

- Passwords
- Keystrokes
- Clipboard contents
- Audio
- Video
- Hidden windows
- Background windows

## Success Criteria

The MVP is successful when:

- A user can record a 10-step desktop workflow.
- Every step has a screenshot.
- The user can edit step descriptions.
- The user can remove unwanted steps.
- The user can export a clean Word document.
- The user can export a clean PDF.
- The app does not feel confusing.
- The generated document looks professional enough to send to a coworker, customer, or developer.
