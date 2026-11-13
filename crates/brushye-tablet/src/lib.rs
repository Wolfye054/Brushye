//! Pen tablet input for Brushye (DECISIONS D-015).
//!
//! Every platform backend produces the same ordered stream of [`PenEvent`]s, so nothing above
//! this crate sees platform details:
//!
//! | Platform | Backend | Delivery |
//! |---|---|---|
//! | Linux/BSD, Wayland | tablet protocol (`zwp_tablet_manager_v2`) on the window's own connection | [`Delivery::Full`] (not yet tested with a real pen) |
//! | Linux/BSD, X11 or Xwayland | XInput2 on a second X connection | [`Delivery::PressureOnly`] (verified with the virtual stylus, D-003) |
//! | Windows | planned: Windows Ink (default), WinTab (option) | — |
//! | macOS | planned: AppKit tablet events | — |
//!
//! [`Delivery`] tells the app what the OS still does on its own: with `Full`, the pen no longer
//! reaches the app as a mouse, so the app must feed the pen into its UI itself; with
//! `PressureOnly`, the pen still arrives as the mouse and the app must inject nothing.
//!
//! Positions are in **window logical points** (top-left origin), sub-pixel. Timestamps are the
//! OS event time in microseconds, on the backend's own clock (only differences are meaningful).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Windows/macOS backends are not written yet: there only the "unsupported" path compiles.
#![cfg_attr(
    not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
    allow(dead_code)
)]

use std::fmt;
use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

pub mod tracker;

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod wayland;
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod x11;

pub use tracker::{Observation, PenTracker};

/// Which end or kind of tool is in proximity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
    Other,
}

/// One pen reading. Axes a backend or device can't report are `None`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenPose {
    /// Window logical points, sub-pixel.
    pub x: f64,
    pub y: f64,
    /// Normalized tip pressure 0..=1 (0 while hovering).
    pub pressure: Option<f32>,
    /// Tilt in degrees, -90..=90 (+x to the right, +y towards the user).
    pub tilt: Option<[f32; 2]>,
    /// Barrel rotation in degrees, 0..360 clockwise.
    pub twist: Option<f32>,
    /// OS event time in microseconds.
    pub time_us: u64,
}

/// The ordered pen event stream. Guarantees (see [`PenTracker`]): `In` comes before any `Pose`,
/// `Down`/`Up` alternate, `Up` is sent before `Out`, and each `Pose` carries every axis the
/// device has (the last known value when an update only changed some axes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenEvent {
    In {
        tool: Tool,
    },
    Pose(PenPose),
    Down,
    Up,
    /// Barrel button (Linux input button code, e.g. `BTN_STYLUS` 0x14b).
    Button {
        button: u32,
        pressed: bool,
    },
    Out,
}

/// What the OS still does with the pen on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// The pen does not reach the app as a mouse: drive the UI from these events too.
    Full,
    /// The pen still arrives as the mouse: only add pressure/tilt; inject nothing.
    PressureOnly,
}

#[derive(Debug)]
pub enum TabletError {
    /// No backend for this platform/window system yet.
    Unsupported(&'static str),
    /// The window system has no tablet (protocol or device) available.
    NotAvailable(String),
    /// Connecting to the window system failed.
    Connection(String),
}

impl fmt::Display for TabletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(s) => write!(f, "tablet input not supported here: {s}"),
            Self::NotAvailable(s) => write!(f, "no tablet available: {s}"),
            Self::Connection(s) => write!(f, "tablet backend connection failed: {s}"),
        }
    }
}

impl std::error::Error for TabletError {}

/// Called (from any thread) when new pen events are ready, e.g. to request a repaint.
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// Shared state between a backend's reader thread and [`Tablet::poll`].
#[derive(Default)]
pub(crate) struct Shared {
    pub(crate) events: std::sync::Mutex<Vec<PenEvent>>,
    pub(crate) waker: std::sync::Mutex<Option<Waker>>,
    pub(crate) scale: std::sync::Mutex<f64>,
}

impl Shared {
    pub(crate) fn new() -> Arc<Self> {
        let s = Self::default();
        if let Ok(mut sc) = s.scale.lock() {
            *sc = 1.0;
        }
        Arc::new(s)
    }

    pub(crate) fn push(&self, evs: impl IntoIterator<Item = PenEvent>) {
        let mut any = false;
        if let Ok(mut q) = self.events.lock() {
            for e in evs {
                q.push(e);
                any = true;
            }
        }
        if any {
            if let Ok(w) = self.waker.lock() {
                if let Some(w) = w.as_ref() {
                    w();
                }
            }
        }
    }

    pub(crate) fn scale(&self) -> f64 {
        self.scale.lock().map(|s| *s).unwrap_or(1.0)
    }
}

/// A running tablet backend for one window.
pub struct Tablet {
    shared: Arc<Shared>,
    delivery: Delivery,
    name: &'static str,
    // Keeps the backend (and its reader thread) alive.
    _backend: Box<dyn Send>,
}

impl Tablet {
    /// Start the backend matching the window's display server.
    pub fn new(
        window: &impl HasWindowHandle,
        display: &impl HasDisplayHandle,
    ) -> Result<Self, TabletError> {
        let wh = window
            .window_handle()
            .map_err(|e| TabletError::Connection(e.to_string()))?
            .as_raw();
        let dh = display
            .display_handle()
            .map_err(|e| TabletError::Connection(e.to_string()))?
            .as_raw();
        Self::from_raw(wh, dh)
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    fn from_raw(_wh: RawWindowHandle, _dh: RawDisplayHandle) -> Result<Self, TabletError> {
        Err(TabletError::Unsupported(
            "no backend for this platform yet (Windows Ink/WinTab and macOS AppKit are planned, D-015)",
        ))
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    fn from_raw(wh: RawWindowHandle, dh: RawDisplayHandle) -> Result<Self, TabletError> {
        let shared = Shared::new();
        match (wh, dh) {
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawWindowHandle::Wayland(_), RawDisplayHandle::Wayland(d)) => {
                let b = wayland::start(d.display.as_ptr(), shared.clone())?;
                Ok(Self {
                    shared,
                    delivery: Delivery::Full,
                    name: "wayland-tablet-v2",
                    _backend: Box::new(b),
                })
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawWindowHandle::Xlib(w), _) => {
                let b = x11::start(
                    u32::try_from(w.window)
                        .map_err(|_| TabletError::Connection("window id".into()))?,
                    shared.clone(),
                )?;
                Ok(Self {
                    shared,
                    delivery: Delivery::PressureOnly,
                    name: "x11-xinput2",
                    _backend: Box::new(b),
                })
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawWindowHandle::Xcb(w), _) => {
                let b = x11::start(w.window.get(), shared.clone())?;
                Ok(Self {
                    shared,
                    delivery: Delivery::PressureOnly,
                    name: "x11-xinput2",
                    _backend: Box::new(b),
                })
            }
            _ => Err(TabletError::Unsupported(
                "no backend for this platform yet (Windows and macOS are planned)",
            )),
        }
    }

    pub fn delivery(&self) -> Delivery {
        self.delivery
    }

    /// Short backend name for logs ("wayland-tablet-v2", "x11-xinput2").
    pub fn backend(&self) -> &'static str {
        self.name
    }

    /// Set the window's scale factor (physical pixels per logical point). Backends that see
    /// physical coordinates divide by it.
    pub fn set_scale_factor(&self, scale: f64) {
        if let Ok(mut s) = self.shared.scale.lock() {
            *s = if scale.is_finite() && scale > 0.0 {
                scale
            } else {
                1.0
            };
        }
    }

    /// Called whenever new events arrive (from the backend's thread).
    pub fn set_waker(&self, waker: Waker) {
        if let Ok(mut w) = self.shared.waker.lock() {
            *w = Some(waker);
        }
    }

    /// Move all pending events, in order, into `out`. Never blocks.
    pub fn poll(&self, out: &mut Vec<PenEvent>) {
        if let Ok(mut q) = self.shared.events.lock() {
            out.append(&mut q);
        }
    }
}
