//! X11 / Xwayland backend: XInput2 on a second, pure-Rust X connection (x11rb). No `unsafe`.
//!
//! - Positions, pressure and tilt: `XI_Motion` events selected on the app window. They carry
//!   sub-pixel window coordinates (`event_x/y`, FP16.16) and the source device's changed
//!   valuators. Several clients may select motion on the same window.
//! - Tip and barrel buttons: `XI_RawButtonPress/Release` on the root window. Only one client
//!   may select (non-raw) button presses on a window, and winit already does.
//! - Stylus devices are slave pointers with an "Abs Pressure" valuator. Xorg's libinput driver
//!   creates them on the pen's first proximity, so the device list is rescanned on
//!   `XI_HierarchyChanged`.
//! - XInput2 has no proximity events: `In` is inferred from the first stylus event, `Out` from
//!   the next event of a non-stylus pointer.
//! - X server timestamps are milliseconds; `time_us` is that × 1000.
//! - Observed with Xorg's libinput driver and the virtual stylus (2026-11-13): right after
//!   proximity-in, X reports the pen's *previous* position twice (once exactly, once rounded
//!   to whole pixels, the server's cursor position) before the real one. These are hover poses
//!   before `Down`, so strokes are unaffected; the very first proximity after startup reports the
//!   cursor's initial position instead.
//!
//! Delivery is pressure-only: the pen still moves the X pointer, so the app gets it as a mouse.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xinput::{self, ConnectionExt as _};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

use crate::{Observation, PenEvent, PenTracker, Shared, TabletError, Tool};

const BTN_STYLUS: u32 = 0x14b;
const BTN_STYLUS2: u32 = 0x14c;

#[derive(Clone, Copy, Debug, Default)]
struct Axis {
    number: u16,
    min: f64,
    max: f64,
}

#[derive(Clone, Debug)]
struct Stylus {
    tool: Tool,
    pressure: Axis,
    tilt_x: Option<Axis>,
    tilt_y: Option<Axis>,
    twist: Option<Axis>,
}

/// Owns the reader thread; dropping it detaches the thread (it ends with the X connection).
pub(crate) struct X11Backend {
    _thread: thread::JoinHandle<()>,
}

fn fp3232(v: xinput::Fp3232) -> f64 {
    f64::from(v.integral) + f64::from(v.frac) / 4_294_967_296.0
}

fn conn_err(e: impl std::fmt::Display) -> TabletError {
    TabletError::Connection(e.to_string())
}

pub(crate) fn start(window: u32, shared: Arc<Shared>) -> Result<X11Backend, TabletError> {
    let (conn, screen_num) = x11rb::connect(None).map_err(conn_err)?;
    let root = conn
        .setup()
        .roots
        .get(screen_num)
        .map(|s| s.root)
        .ok_or_else(|| conn_err("no X screen"))?;
    let ver = conn
        .xinput_xi_query_version(2, 2)
        .map_err(conn_err)?
        .reply()
        .map_err(conn_err)?;
    if (ver.major_version, ver.minor_version) < (2, 2) {
        return Err(TabletError::NotAvailable(format!(
            "XInput {}.{} < 2.2",
            ver.major_version, ver.minor_version
        )));
    }
    let all = xinput::Device::ALL;
    conn.xinput_xi_select_events(
        window,
        &[xinput::EventMask {
            deviceid: all.into(),
            mask: vec![xinput::XIEventMask::MOTION],
        }],
    )
    .map_err(conn_err)?
    .check()
    .map_err(conn_err)?;
    conn.xinput_xi_select_events(
        root,
        &[xinput::EventMask {
            deviceid: all.into(),
            mask: vec![
                xinput::XIEventMask::RAW_BUTTON_PRESS
                    | xinput::XIEventMask::RAW_BUTTON_RELEASE
                    | xinput::XIEventMask::HIERARCHY,
            ],
        }],
    )
    .map_err(conn_err)?
    .check()
    .map_err(conn_err)?;
    let styluses = scan(&conn).map_err(conn_err)?;
    let thread = thread::Builder::new()
        .name("brushye-tablet-x11".into())
        .spawn(move || run(conn, styluses, shared))
        .map_err(conn_err)?;
    Ok(X11Backend { _thread: thread })
}

/// Find slave pointers that have an "Abs Pressure" valuator.
fn scan(conn: &RustConnection) -> Result<HashMap<u16, Stylus>, Box<dyn std::error::Error>> {
    let reply = conn.xinput_xi_query_device(xinput::Device::ALL)?.reply()?;
    let mut names: HashMap<u32, String> = HashMap::new();
    let mut out = HashMap::new();
    for info in reply.infos {
        if !matches!(
            info.type_,
            xinput::DeviceType::SLAVE_POINTER | xinput::DeviceType::FLOATING_SLAVE
        ) {
            continue;
        }
        let mut axes: HashMap<String, Axis> = HashMap::new();
        for class in &info.classes {
            if let xinput::DeviceClassData::Valuator(v) = &class.data {
                let label = match names.get(&v.label) {
                    Some(n) => n.clone(),
                    None if v.label == 0 => String::new(),
                    None => {
                        let n =
                            String::from_utf8_lossy(&conn.get_atom_name(v.label)?.reply()?.name)
                                .into_owned();
                        names.insert(v.label, n.clone());
                        n
                    }
                };
                axes.insert(
                    label,
                    Axis {
                        number: v.number,
                        min: fp3232(v.min),
                        max: fp3232(v.max),
                    },
                );
            }
        }
        let Some(pressure) = axes.get("Abs Pressure").copied() else {
            continue;
        };
        let name = String::from_utf8_lossy(&info.name).to_lowercase();
        let tool = if name.contains("eraser") {
            Tool::Eraser
        } else {
            Tool::Pen
        };
        out.insert(
            info.deviceid,
            Stylus {
                tool,
                pressure,
                tilt_x: axes.get("Abs Tilt X").copied(),
                tilt_y: axes.get("Abs Tilt Y").copied(),
                twist: axes.get("Abs Rotary Z").copied(),
            },
        );
    }
    Ok(out)
}

/// Values of the valuators present in `mask`, by valuator number.
fn valuators(mask: &[u32], values: &[xinput::Fp3232]) -> HashMap<u16, f64> {
    let mut out = HashMap::new();
    let mut i = 0usize;
    for (word, bits) in mask.iter().enumerate() {
        for bit in 0..32u32 {
            if bits & (1 << bit) != 0 {
                if let Some(v) = values.get(i) {
                    let n = u16::try_from(word * 32)
                        .unwrap_or(u16::MAX)
                        .saturating_add(bit as u16);
                    out.insert(n, fp3232(*v));
                }
                i += 1;
            }
        }
    }
    out
}

fn normalized(a: Axis, v: f64) -> f32 {
    let span = a.max - a.min;
    if span > 0.0 {
        ((v - a.min) / span).clamp(0.0, 1.0) as f32
    } else {
        0.0
    }
}

/// Map one stylus motion event to an observation (pure; unit-tested).
fn observation(
    s: &Stylus,
    event_x: i32,
    event_y: i32,
    mask: &[u32],
    values: &[xinput::Fp3232],
    time_ms: u32,
    scale: f64,
) -> Observation {
    let v = valuators(mask, values);
    let get = |a: Option<Axis>| a.and_then(|a| v.get(&a.number).copied());
    Observation {
        tool: Some(s.tool),
        x: Some(f64::from(event_x) / 65536.0 / scale),
        y: Some(f64::from(event_y) / 65536.0 / scale),
        pressure: v
            .get(&s.pressure.number)
            .map(|p| normalized(s.pressure, *p)),
        // Tilt valuators are in degrees on the libinput and wacom X drivers.
        tilt_x: get(s.tilt_x).map(|t| t as f32),
        tilt_y: get(s.tilt_y).map(|t| t as f32),
        twist: get(s.twist).map(|t| t as f32),
        time_us: Some(u64::from(time_ms) * 1000),
    }
}

fn run(conn: RustConnection, mut styluses: HashMap<u16, Stylus>, shared: Arc<Shared>) {
    let mut tracker = PenTracker::new();
    let mut out: Vec<PenEvent> = Vec::new();
    while let Ok(ev) = conn.wait_for_event() {
        match ev {
            Event::XinputMotion(m) => {
                if let Some(s) = styluses.get(&m.sourceid) {
                    // Slave events carry the device's own valuators; the master copy is skipped.
                    if m.deviceid == m.sourceid {
                        tracker.observe(
                            observation(
                                s,
                                m.event_x,
                                m.event_y,
                                &m.valuator_mask,
                                &m.axisvalues,
                                m.time,
                                shared.scale(),
                            ),
                            &mut out,
                        );
                    }
                } else if m.deviceid == m.sourceid && tracker.tool().is_some() {
                    tracker.proximity_out(&mut out);
                }
            }
            Event::XinputRawButtonPress(b) | Event::XinputRawButtonRelease(b) => {
                if styluses.contains_key(&b.sourceid) {
                    let pressed = b.event_type == xinput::RAW_BUTTON_PRESS_EVENT;
                    match b.detail {
                        1 => tracker.tip(pressed, &mut out),
                        2 => tracker.button(BTN_STYLUS, pressed, &mut out),
                        3 => tracker.button(BTN_STYLUS2, pressed, &mut out),
                        _ => {}
                    }
                }
            }
            Event::XinputHierarchy(_) => {
                if let Ok(s) = scan(&conn) {
                    styluses = s;
                }
            }
            _ => {}
        }
        if !out.is_empty() {
            shared.push(out.drain(..));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(v: f64) -> xinput::Fp3232 {
        xinput::Fp3232 {
            integral: v.floor() as i32,
            frac: ((v - v.floor()) * 4_294_967_296.0) as u32,
        }
    }

    fn stylus() -> Stylus {
        Stylus {
            tool: Tool::Pen,
            pressure: Axis {
                number: 2,
                min: 0.0,
                max: 2047.0,
            },
            tilt_x: Some(Axis {
                number: 3,
                min: -64.0,
                max: 64.0,
            }),
            tilt_y: Some(Axis {
                number: 4,
                min: -64.0,
                max: 64.0,
            }),
            twist: None,
        }
    }

    #[test]
    fn maps_subpixel_position_pressure_and_tilt() {
        // mask: valuators 2, 3, 4 present
        let o = observation(
            &stylus(),
            (100 << 16) + (1 << 15),
            50 << 16,
            &[0b11100],
            &[fp(1023.5), fp(-10.0), fp(20.0)],
            1234,
            1.0,
        );
        assert_eq!(o.x, Some(100.5));
        assert_eq!(o.y, Some(50.0));
        assert!((o.pressure.unwrap_or(0.0) - 0.5).abs() < 1e-4);
        assert_eq!(o.tilt_x, Some(-10.0));
        assert_eq!(o.tilt_y, Some(20.0));
        assert_eq!(o.time_us, Some(1_234_000));
    }

    #[test]
    fn missing_valuators_are_none_and_scale_is_applied() {
        let o = observation(
            &stylus(),
            200 << 16,
            100 << 16,
            &[0b01000],
            &[fp(5.0)],
            0,
            2.0,
        );
        assert_eq!((o.x, o.y), (Some(100.0), Some(50.0)));
        assert_eq!(o.pressure, None);
        assert_eq!(o.tilt_x, Some(5.0));
        assert_eq!(o.tilt_y, None);
    }

    #[test]
    fn valuator_numbers_follow_mask_bits_across_words() {
        let v = valuators(&[0b1, 0b10], &[fp(1.0), fp(2.0)]);
        assert_eq!(v.get(&0), Some(&1.0));
        assert_eq!(v.get(&33), Some(&2.0));
    }
}
