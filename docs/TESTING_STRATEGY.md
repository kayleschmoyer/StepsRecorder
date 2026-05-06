# Testing Strategy

## Testing Goal

Testing must prove that the app can reliably record, review, edit, and export desktop workflow documentation.

The app touches native Windows behavior, file storage, screenshots, export generation, and user-facing editing. Testing must cover both UI behavior and native backend logic.

## Test Types

## Unit Tests

Use unit tests for:

- Date formatting
- Step numbering
- Reordering logic
- Export option validation
- Settings parsing
- Error mapping
- File path helpers
- Click debounce logic
- Document model generation

Frontend:

- Vitest
- React Testing Library

Rust:

- Rust unit tests

## Component Tests

Use component tests for:

- Start recording button state
- Recording status panel
- Session list
- Step card
- Step editor
- Export dialog
- Empty states
- Error states
- Screenshot editor toolbar

## Integration Tests

Use integration tests for:

- Creating a session
- Adding step metadata
- Updating step text
- Reordering steps
- Deleting steps
- Saving settings
- Building document model
- Exporting a sample DOCX
- Exporting a sample PDF

## End-to-End Tests

Use Playwright for app-level workflows where feasible.

Critical E2E flows:

1. User opens app and sees home screen.
2. User starts a recording.
3. Recording status is visible.
4. User stops recording.
5. User lands on review screen.
6. User edits a step.
7. User deletes a step.
8. User exports DOCX.
9. User exports PDF.
10. User sees success message.

Native click capture may require a controlled test harness or manual QA for full validation.

## Manual QA Requirements

Manual QA is required for Windows-level behavior.

Manual scenarios:

### Scenario 1: Basic Recording

1. Open StepForge Recorder.
2. Click Start Recording.
3. Open Notepad.
4. Click inside Notepad.
5. Click the File menu.
6. Click Save As.
7. Stop recording.
8. Confirm steps were created.
9. Confirm screenshots match visible screen state.

### Scenario 2: Multi-Monitor Recording

1. Use two monitors.
2. Start recording.
3. Click on monitor 1.
4. Click on monitor 2.
5. Stop recording.
6. Confirm each screenshot captured the correct monitor.

### Scenario 3: Rapid Clicking

1. Start recording.
2. Rapidly click the same button multiple times.
3. Stop recording.
4. Confirm duplicate spam is reduced based on debounce rules.

### Scenario 4: Edit Steps

1. Record at least five steps.
2. Edit titles.
3. Edit descriptions.
4. Delete one step.
5. Reorder two steps.
6. Confirm final order is correct.

### Scenario 5: Screenshot Editing

1. Open a captured screenshot.
2. Add a redaction rectangle.
3. Crop the image.
4. Save edits.
5. Confirm edited screenshot appears in review.
6. Export and confirm edited screenshot is used.

### Scenario 6: DOCX Export

1. Record a workflow.
2. Edit step descriptions.
3. Export DOCX.
4. Open in Microsoft Word.
5. Confirm formatting, screenshots, and step order.

### Scenario 7: PDF Export

1. Record a workflow.
2. Export PDF.
3. Open PDF.
4. Confirm formatting, screenshots, and step order.

### Scenario 8: Missing Screenshot File

1. Record a session.
2. Manually delete one screenshot file.
3. Attempt export.
4. Confirm friendly error explains the issue.

### Scenario 9: Privacy Reminder

1. Record session.
2. Click export.
3. Confirm privacy reminder appears if setting is enabled.
4. Confirm user must acknowledge before export.

## Acceptance Criteria

The MVP passes testing when:

- Recording starts and stops reliably.
- Clicks create steps.
- Screenshots are captured for each step.
- The correct monitor is captured in multi-monitor setups.
- Review screen shows all active steps in order.
- Step edits persist after app restart.
- Deleted steps do not export.
- Reordered steps export in correct order.
- DOCX export opens successfully.
- PDF export opens successfully.
- Errors are friendly and actionable.
- The app does not crash during normal recording.

## Regression Test Checklist

Before release, verify:

- App opens.
- Start Recording works.
- Stop Recording works.
- Session review loads.
- Step editing works.
- Screenshot preview works.
- Screenshot editing works.
- DOCX export works.
- PDF export works.
- Settings persist.
- App restarts cleanly.
- Existing sessions still load.
