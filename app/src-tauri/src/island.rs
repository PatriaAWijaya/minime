//! Dynamic Island — a real always-on-top floating window that mirrors
//! agent status at the top (or bottom) of the screen. The main window
//! composes an [`IslandPayload`]; `island_sync` applies visibility, size
//! and position per monitor and forwards the state to every island
//! window through the `island-state` event.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, State, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandPayload {
    pub notch: bool,
    pub idle: bool,
    pub extra_displays: bool,
    pub haptics: bool,
    pub position: String,
    pub width_pct: f64,
    pub height_pct: f64,
    /// "status" | "idle" | "off"
    pub mode: String,
    pub title: String,
    pub sub: String,
    pub attention: bool,
    pub logo_color: String,
}

/// Last payload, so a freshly loaded island window can fetch its state.
pub struct IslandState(pub Mutex<Option<IslandPayload>>);

#[derive(Serialize)]
pub struct IslandSyncResult {
    pub mode: String,
    pub displays: u32,
}

/// Height of the macOS menu bar in physical pixels (0 on other platforms).
/// A window placed at y < menu_bar_height would be hidden behind the menu
/// bar, so the island sits just below it — visually merged with the bar.
#[allow(unexpected_cfgs)]
fn menu_bar_height() -> i32 {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::{class, msg_send, sel, sel_impl};

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct NsPoint {
            x: f64,
            y: f64,
        }
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct NsSize {
            width: f64,
            height: f64,
        }
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct NsRect {
            origin: NsPoint,
            size: NsSize,
        }

        let screen: *mut objc::runtime::Object = msg_send![class!(NSScreen), mainScreen];
        if screen.is_null() {
            return 0;
        }
        let frame: NsRect = msg_send![screen, frame];
        let visible: NsRect = msg_send![screen, visibleFrame];
        // visibleFrame excludes the menu bar (top) and the Dock (bottom/side).
        // Top gap = full height - visible height - visible origin y (Dock).
        let scale: f64 = msg_send![screen, backingScaleFactor];
        let top_gap = (frame.size.height - visible.size.height) - visible.origin.y;
        if top_gap > 0.0 {
            return (top_gap * scale).round() as i32;
        }
        // Fallback: standard menu bar height.
        (24.0 * scale).round() as i32
    }
    #[cfg(not(target_os = "macos"))]
    {
        0
    }
}

fn apply_to_window(window: &WebviewWindow, monitor: &tauri::Monitor, payload: &IslandPayload) {
    let width = (400.0 * payload.width_pct / 100.0).clamp(280.0, 640.0);
    let height = (56.0 * payload.height_pct / 100.0).clamp(40.0, 110.0);
    let _ = window.set_size(LogicalSize::new(width, height));
    if let Ok(outer) = window.outer_size() {
        let mpos = monitor.position();
        let msize = monitor.size();
        let x = mpos.x + (msize.width as i32 - outer.width as i32) / 2;
        // "Top" = tucked right under the desktop top bar (macOS menu bar),
        // so the pill reads as part of the system chrome (notch-style).
        let top_bar = menu_bar_height();
        let y = if payload.position == "bottom" {
            mpos.y + msize.height as i32 - outer.height as i32 - 16
        } else {
            mpos.y + top_bar + 6
        };
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    let _ = window.emit("island-state", payload.clone());
    let _ = window.show();
}

/// Apply the island payload: show/hide/position the island window on the
/// primary display, spawn one island per additional display when asked,
/// close extra islands that are no longer wanted, and remember the
/// payload for `island_get_state`.
#[tauri::command]
pub fn island_sync(
    app: AppHandle,
    state: State<'_, IslandState>,
    payload: IslandPayload,
) -> Result<IslandSyncResult, String> {
    if let Ok(mut guard) = state.0.lock() {
        *guard = Some(payload.clone());
    }
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let primary = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| monitors.first().cloned());
    let visible = payload.mode != "off";
    let mut displays: u32 = 0;

    // Primary display: the config-created "island" window.
    if let (Some(win), Some(mon)) = (app.get_webview_window("island"), primary.clone()) {
        if visible {
            apply_to_window(&win, &mon, &payload);
            displays += 1;
        } else {
            let _ = win.hide();
        }
    }

    // Additional displays: every monitor that is not the primary one.
    let extras: Vec<tauri::Monitor> = match &primary {
        Some(p) => monitors
            .iter()
            .filter(|m| m.position() != p.position())
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    let want_extra = if visible && payload.extra_displays {
        extras.len()
    } else {
        0
    };
    for (i, mon) in extras.iter().enumerate() {
        let label = format!("island-x{}", i + 1);
        let existing = app.get_webview_window(&label);
        if i < want_extra {
            let win = match existing {
                Some(w) => w,
                None => WebviewWindowBuilder::new(
                    &app,
                    &label,
                    WebviewUrl::App("index.html?view=island".into()),
                )
                .title("Minime Island")
                .inner_size(400.0, 56.0)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .shadow(false)
                .visible(false)
                .build()
                .map_err(|e| e.to_string())?,
            };
            apply_to_window(&win, mon, &payload);
            displays += 1;
        } else if let Some(w) = existing {
            let _ = w.close();
        }
    }

    Ok(IslandSyncResult {
        mode: payload.mode.clone(),
        displays,
    })
}

/// The last payload pushed by the main window (island windows fetch this
/// on mount so they never miss the current state).
#[tauri::command]
pub fn island_get_state(state: State<'_, IslandState>) -> Option<IslandPayload> {
    state.0.lock().ok().and_then(|g| g.clone())
}

/// Fire a macOS Force Touch haptic (generic pattern). No-op elsewhere —
/// other platforms have no system haptic API for desktop windows.
// The objc 0.2 macros expand with an internal cfg(cargo-clippy) check;
// silence that crate-generated lint at this single call site.
#[allow(unexpected_cfgs)]
#[tauri::command]
pub fn island_haptic() {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::{class, msg_send, sel, sel_impl};
        let performer: *mut objc::runtime::Object =
            msg_send![class!(NSHapticFeedbackManager), defaultPerformer];
        if !performer.is_null() {
            // NSHapticFeedbackPatternGeneric = 0,
            // NSHapticFeedbackPerformanceTimeDefault = 0.
            let _: () = msg_send![performer, performFeedbackPattern: 0usize performanceTime: 0usize];
        }
    }
}
