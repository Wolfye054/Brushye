//! Wayland backend: the tablet protocol (`zwp_tablet_manager_v2`) on the app window's own
//! Wayland connection. Written from the protocol specification (tablet-unstable-v2).
//!
//! - Tablet events name the client's `wl_surface`, so they must be read on the same connection
//!   as the window. We wrap the app's `wl_display` (from its display handle) in a separate
//!   event queue. A reader thread calls `blocking_dispatch` on that queue, which uses libwayland's
//!   multi-reader protocol (prepare-read / read), so it coexists with winit's own reading.
//! - Protocol versions are bound at min(compositor, client). GNOME 46+ advertises tablet v2;
//!   binding the advertised version with v1-only handling breaks tablet detection.
//! - Once the tablet seat is bound, the compositor stops emulating a pointer for the pen
//!   (delivery [`crate::Delivery::Full`]): the app must feed the pen into its UI itself.
//! - Positions are surface-local logical coordinates; times are milliseconds × 1000.

use std::sync::{Arc, Mutex};
use std::thread;

use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, WEnum, event_created_child};
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2, zwp_tablet_pad_group_v2,
    zwp_tablet_pad_group_v2::ZwpTabletPadGroupV2, zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2,
    zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2, zwp_tablet_pad_v2,
    zwp_tablet_pad_v2::ZwpTabletPadV2, zwp_tablet_seat_v2, zwp_tablet_seat_v2::ZwpTabletSeatV2,
    zwp_tablet_tool_v2, zwp_tablet_tool_v2::ZwpTabletToolV2, zwp_tablet_v2::ZwpTabletV2,
};

use crate::{Observation, PenEvent, PenTracker, Shared, TabletError, Tool};

const BTN_STYLUS: u32 = 0x14b;

/// Highest protocol versions this code handles.
const SEAT_VERSION: u32 = 1;
const TABLET_MANAGER_VERSION: u32 = 1;

pub(crate) struct WaylandBackend {
    _thread: thread::JoinHandle<()>,
}

/// Per-tool user data: the tool type arrives before the tool is usable.
#[derive(Default)]
struct ToolData {
    kind: Mutex<Option<Tool>>,
}

/// Axis values and transitions collected until the tool's `frame` event.
#[derive(Default)]
struct Pending {
    obs: Observation,
    any_axis: bool,
    down: Option<bool>,
    buttons: Vec<(u32, bool)>,
}

struct State {
    shared: Arc<Shared>,
    tracker: PenTracker,
    out: Vec<PenEvent>,
    pending: Pending,
}

fn tool_kind(t: WEnum<zwp_tablet_tool_v2::Type>) -> Tool {
    use zwp_tablet_tool_v2::Type as T;
    match t {
        WEnum::Value(T::Pen | T::Pencil | T::Brush | T::Airbrush) => Tool::Pen,
        WEnum::Value(T::Eraser) => Tool::Eraser,
        _ => Tool::Other,
    }
}

pub(crate) fn start(
    display: *mut std::ffi::c_void,
    shared: Arc<Shared>,
) -> Result<WaylandBackend, TabletError> {
    #[allow(unsafe_code)]
    // SAFETY: `display` comes from the window's live raw display handle (a `wl_display*` owned by
    // the windowing library, which outlives the window and therefore this backend's use of it).
    // `from_foreign_display` does not take ownership: it only creates a client-side view that
    // uses its own event queues, which libwayland supports alongside the owner's queues.
    let backend = unsafe { wayland_backend::client::Backend::from_foreign_display(display.cast()) };
    let conn = Connection::from_backend(backend);
    let (globals, mut queue) = wayland_client::globals::registry_queue_init::<State>(&conn)
        .map_err(|e| TabletError::Connection(e.to_string()))?;
    let qh = queue.handle();
    let seat: wl_seat::WlSeat = globals
        .bind(&qh, 1..=SEAT_VERSION, ())
        .map_err(|e| TabletError::NotAvailable(format!("wl_seat: {e}")))?;
    let manager: ZwpTabletManagerV2 =
        globals
            .bind(&qh, 1..=TABLET_MANAGER_VERSION, ())
            .map_err(|e| {
                TabletError::NotAvailable(format!(
                    "compositor has no tablet protocol (zwp_tablet_manager_v2): {e}"
                ))
            })?;
    let _tablet_seat = manager.get_tablet_seat(&seat, &qh, ());
    let mut state = State {
        shared,
        tracker: PenTracker::new(),
        out: Vec::new(),
        pending: Pending::default(),
    };
    // Process the initial tablet/tool announcements before returning.
    queue
        .roundtrip(&mut state)
        .map_err(|e| TabletError::Connection(e.to_string()))?;
    let thread = thread::Builder::new()
        .name("brushye-tablet-wayland".into())
        .spawn(move || {
            // Keep the proxies alive for the thread's lifetime.
            let _keep = (seat, manager, _tablet_seat, conn);
            while queue.blocking_dispatch(&mut state).is_ok() {}
        })
        .map_err(|e| TabletError::Connection(e.to_string()))?;
    Ok(WaylandBackend { _thread: thread })
}

impl State {
    fn flush(&mut self) {
        let p = std::mem::take(&mut self.pending);
        if p.down == Some(false) {
            // Report the lift after the frame's last position.
            if p.any_axis {
                self.tracker.observe(p.obs, &mut self.out);
            }
            self.tracker.tip(false, &mut self.out);
        } else {
            if p.any_axis {
                self.tracker.observe(p.obs, &mut self.out);
            }
            if p.down == Some(true) {
                self.tracker.tip(true, &mut self.out);
            }
        }
        for (b, pressed) in p.buttons {
            self.tracker.button(b, pressed, &mut self.out);
        }
        if !self.out.is_empty() {
            self.shared.push(self.out.drain(..));
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletManagerV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletManagerV2,
        _: <ZwpTabletManagerV2 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletSeatV2,
        _: zwp_tablet_seat_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }

    event_created_child!(State, ZwpTabletSeatV2, [
        zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (ZwpTabletV2, ()),
        zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (ZwpTabletToolV2, ToolData::default()),
        zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (ZwpTabletPadV2, ()),
    ]);
}

impl Dispatch<ZwpTabletV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletV2,
        _: <ZwpTabletV2 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletPadV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadV2,
        _: zwp_tablet_pad_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }

    event_created_child!(State, ZwpTabletPadV2, [
        zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (ZwpTabletPadGroupV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadGroupV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadGroupV2,
        _: zwp_tablet_pad_group_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }

    event_created_child!(State, ZwpTabletPadGroupV2, [
        zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (ZwpTabletPadRingV2, ()),
        zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (ZwpTabletPadStripV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadRingV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadRingV2,
        _: <ZwpTabletPadRingV2 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletPadStripV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadStripV2,
        _: <ZwpTabletPadStripV2 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpTabletToolV2, ToolData> for State {
    fn event(
        state: &mut Self,
        _: &ZwpTabletToolV2,
        event: zwp_tablet_tool_v2::Event,
        data: &ToolData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwp_tablet_tool_v2::Event as E;
        let p = &mut state.pending;
        match event {
            E::Type { tool_type } => {
                if let Ok(mut k) = data.kind.lock() {
                    *k = Some(tool_kind(tool_type));
                }
            }
            E::ProximityIn { .. } => {
                let tool = data.kind.lock().ok().and_then(|k| *k).unwrap_or(Tool::Pen);
                state.tracker.proximity_in(tool, &mut state.out);
            }
            E::ProximityOut => {
                state.flush();
                state.tracker.proximity_out(&mut state.out);
                state.shared.push(state.out.drain(..));
            }
            E::Down { .. } => p.down = Some(true),
            E::Up => p.down = Some(false),
            E::Motion { x, y } => {
                p.obs.x = Some(x);
                p.obs.y = Some(y);
                p.any_axis = true;
            }
            E::Pressure { pressure } => {
                p.obs.pressure = Some(pressure as f32 / 65535.0);
                p.any_axis = true;
            }
            E::Tilt { tilt_x, tilt_y } => {
                p.obs.tilt_x = Some(tilt_x as f32);
                p.obs.tilt_y = Some(tilt_y as f32);
                p.any_axis = true;
            }
            E::Rotation { degrees } => {
                p.obs.twist = Some(degrees as f32);
                p.any_axis = true;
            }
            E::Button {
                button, state: s, ..
            } => {
                let pressed = matches!(s, WEnum::Value(zwp_tablet_tool_v2::ButtonState::Pressed));
                // Linux input codes; the primary barrel button is BTN_STYLUS.
                p.buttons
                    .push((if button == 0 { BTN_STYLUS } else { button }, pressed));
            }
            E::Frame { time } => {
                p.obs.time_us = Some(u64::from(time) * 1000);
                state.flush();
            }
            E::Removed => {
                state.tracker.proximity_out(&mut state.out);
                state.shared.push(state.out.drain(..));
            }
            _ => {}
        }
    }
}
