# Performance Requirements

## Performance Goal

The app must feel fast and invisible during recording.

Recording should not slow down the user's workflow inside the target Windows desktop application.

## Startup Performance

Target:

- App shell visible within 2 seconds on a typical Windows 11 workstation.
- Recent sessions loaded after shell render if needed.
- No blocking heavy work on startup.

## Recording Performance

During recording:

- Click detection should feel instant.
- Screenshot capture should happen as close as possible to the click event.
- UI should remain responsive.
- The target application should not lag due to recording.
- Recording 50 steps should not noticeably degrade performance.

## Screenshot Capture Target

Target:

- Capture should complete within 300ms where feasible.
- If capture takes longer, the app should still remain stable.
- Long capture delays should be logged for diagnostics.

## Storage Performance

- Screenshots should be written efficiently.
- Avoid storing full images in SQLite.
- Generate thumbnails if large screenshots make the review UI slow.
- Lazy-load full screenshots in review/edit screens.

## Review Screen Performance

The review screen should handle:

- 50 steps smoothly.
- 100 steps acceptably.
- Large screenshots without freezing.

Recommended:

- Use thumbnail previews in lists.
- Load full screenshot only for selected step.
- Avoid rendering every full-size screenshot at once.

## Export Performance

Export should support:

- 50-step DOCX
- 50-step PDF

Export should show progress if it takes more than 2 seconds.

The app should not appear frozen during export.

## Memory Requirements

Avoid holding all full-size screenshots in memory at once.

Use file paths and load images as needed.

## Debounce Requirement

Default click debounce:

```text
500ms
```

Purpose:

- Prevent accidental duplicate steps from rapid clicking.
- Reduce screenshot spam.
- Keep recording output clean.

## Logging Performance

Logs should be lightweight.

Do not log excessively during every mouse move.

For MVP, only click events matter.

## Performance Anti-Patterns

Do not:

- Store image blobs in SQLite.
- Load all full-size screenshots into React state.
- Re-render the full review screen on every small field edit.
- Generate exports from visual DOM scraping.
- Run expensive OCR/AI during MVP recording.
- Block UI while writing large files.
