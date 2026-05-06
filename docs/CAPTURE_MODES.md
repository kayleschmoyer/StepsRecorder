# Screenshot Capture Modes

Steps Recorder currently supports two native screenshot capture modes. The selected mode is stored in the existing `app_settings.screenshot_mode` setting and is read by the Rust capture worker when each accepted click is persisted.

## `clicked_monitor`

`clicked_monitor` is the safe fallback and preserves the existing behavior: the app resolves the monitor nearest to the click point and captures that monitor's full rectangle.

## `clicked_window`

`clicked_window` is implemented for Windows. When the user clicks inside a desktop application, the Rust capture code resolves the top-level window under the click point and captures that window's visible screen rectangle instead of the whole monitor.

The captured rectangle includes the window title bar and visible borders because the implementation uses DWM extended frame bounds when available, with `GetWindowRect` as a bounds fallback. It does not use `PrintWindow`, so it does not capture hidden, minimized, or background windows behind the clicked app.

If another foreground window partially covers the clicked window, the screenshot is a visible screen capture of the clicked window rectangle. Covered portions will show the covering foreground window because those pixels are what is visible on screen; pixels from windows behind the clicked application are not revealed.

## Fallback behavior

If `clicked_window` cannot resolve a visible, non-minimized top-level window or cannot capture its rectangle, the app logs the reason and falls back to `clicked_monitor` for that same click. Unknown or missing setting values also default to `clicked_monitor`.

## DPI and multi-monitor behavior

The Windows clicked-window capture temporarily switches the capture thread to per-monitor DPI awareness before resolving the clicked window and reading bounds. Capture coordinates are kept in virtual desktop screen coordinates, which supports multi-monitor layouts and monitors placed at negative X/Y coordinates. The same BitBlt path is used for both modes so marker coordinates are computed relative to the captured rectangle without changing existing step metadata.
