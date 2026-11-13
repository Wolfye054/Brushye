//! Pure pen-state tracking shared by all backends.
//!
//! Backends report raw [`Observation`]s in the order the OS delivers them. The tracker turns
//! them into a well-formed [`PenEvent`] stream:
//! - `In` is emitted before the first `Pose`/`Down` of a tool (also when the backend never saw
//!   an explicit proximity event, as on X11).
//! - Axes an update leaves out keep their last value, so every `Pose` is complete. (Partial
//!   updates are normal: X11 sends only changed valuators, Wayland sends one event per axis.)
//! - `Down`/`Up` alternate; a lost `Up` is synthesised before `Out` or a tool change.
//! - Values are clamped to the documented ranges; non-finite values are dropped.
//! - Barrel rotation (twist) is reported only once it has varied for that tool. Some drivers
//!   declare a rotation axis they never fill: OpenTabletDriver's virtual tablet made GNOME report
//!   a constant 90° for a pen without rotation sensing (2026-11-13). Once a tool's twist has
//!   changed it is reported from then on, also in later proximity sessions.

use crate::{PenEvent, PenPose, Tool};

/// One raw backend observation. Fields that didn't change are `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Observation {
    pub tool: Option<Tool>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub pressure: Option<f32>,
    pub tilt_x: Option<f32>,
    pub tilt_y: Option<f32>,
    pub twist: Option<f32>,
    pub time_us: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Axes {
    x: Option<f64>,
    y: Option<f64>,
    pressure: Option<f32>,
    tilt: [Option<f32>; 2],
    twist: Option<f32>,
    time_us: u64,
}

/// State machine turning [`Observation`]s into the guaranteed [`PenEvent`] order.
#[derive(Debug, Default)]
pub struct PenTracker {
    tool: Option<Tool>,
    down: bool,
    axes: Axes,
    /// First twist value seen per tool, and whether it has varied since (see module docs).
    twist_first: [Option<f32>; 3],
    twist_varied: [bool; 3],
}

/// Twist values closer than this count as unchanged.
const TWIST_EPSILON: f32 = 0.01;

fn tool_index(t: Tool) -> usize {
    match t {
        Tool::Pen => 0,
        Tool::Eraser => 1,
        Tool::Other => 2,
    }
}

fn finite32(v: Option<f32>) -> Option<f32> {
    v.filter(|v| v.is_finite())
}

impl PenTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tool(&self) -> Option<Tool> {
        self.tool
    }

    pub fn is_down(&self) -> bool {
        self.down
    }

    /// The tool entered proximity.
    pub fn proximity_in(&mut self, tool: Tool, out: &mut Vec<PenEvent>) {
        if self.tool == Some(tool) {
            return;
        }
        if self.tool.is_some() {
            self.proximity_out(out);
        }
        self.tool = Some(tool);
        self.axes = Axes::default();
        out.push(PenEvent::In { tool });
    }

    /// The tool left proximity.
    pub fn proximity_out(&mut self, out: &mut Vec<PenEvent>) {
        if self.tool.is_none() {
            return;
        }
        if self.down {
            self.tip(false, out);
        }
        self.tool = None;
        out.push(PenEvent::Out);
    }

    /// Axis update. Emits a `Pose` once a position is known.
    pub fn observe(&mut self, o: Observation, out: &mut Vec<PenEvent>) {
        if let Some(t) = o.tool {
            self.proximity_in(t, out);
        } else if self.tool.is_none() {
            self.proximity_in(Tool::Pen, out);
        }
        let a = &mut self.axes;
        if let Some(x) = o.x.filter(|v| v.is_finite()) {
            a.x = Some(x);
        }
        if let Some(y) = o.y.filter(|v| v.is_finite()) {
            a.y = Some(y);
        }
        if let Some(p) = finite32(o.pressure) {
            a.pressure = Some(p.clamp(0.0, 1.0));
        }
        if let Some(t) = finite32(o.tilt_x) {
            a.tilt[0] = Some(t.clamp(-90.0, 90.0));
        }
        if let Some(t) = finite32(o.tilt_y) {
            a.tilt[1] = Some(t.clamp(-90.0, 90.0));
        }
        if let Some(r) = finite32(o.twist) {
            let r = r.rem_euclid(360.0);
            let r = if r >= 360.0 { 0.0 } else { r };
            a.twist = Some(r);
            let i = tool_index(self.tool.unwrap_or(Tool::Pen));
            match self.twist_first[i] {
                None => self.twist_first[i] = Some(r),
                Some(f) if (f - r).abs() > TWIST_EPSILON => self.twist_varied[i] = true,
                Some(_) => {}
            }
        }
        if let Some(t) = o.time_us {
            a.time_us = t;
        }
        let twist_ok = self.twist_varied[tool_index(self.tool.unwrap_or(Tool::Pen))];
        let a = &self.axes;
        if let (Some(x), Some(y)) = (a.x, a.y) {
            let tilt = match a.tilt {
                [Some(tx), Some(ty)] => Some([tx, ty]),
                [Some(tx), None] => Some([tx, 0.0]),
                [None, Some(ty)] => Some([0.0, ty]),
                [None, None] => None,
            };
            out.push(PenEvent::Pose(PenPose {
                x,
                y,
                pressure: a.pressure,
                tilt,
                twist: if twist_ok { a.twist } else { None },
                time_us: a.time_us,
            }));
        }
    }

    /// Tip contact changed. Duplicate transitions are ignored.
    pub fn tip(&mut self, down: bool, out: &mut Vec<PenEvent>) {
        if self.tool.is_none() && down {
            self.proximity_in(Tool::Pen, out);
        }
        if self.down == down {
            return;
        }
        self.down = down;
        if !down {
            // Hovering has zero pressure.
            if self.axes.pressure.is_some() {
                self.axes.pressure = Some(0.0);
            }
        }
        out.push(if down { PenEvent::Down } else { PenEvent::Up });
    }

    pub fn button(&mut self, button: u32, pressed: bool, out: &mut Vec<PenEvent>) {
        out.push(PenEvent::Button { button, pressed });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(x: f64, y: f64, p: f32) -> Observation {
        Observation {
            x: Some(x),
            y: Some(y),
            pressure: Some(p),
            ..Default::default()
        }
    }

    #[test]
    fn in_is_emitted_before_first_pose() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(obs(1.0, 2.0, 0.5), &mut out);
        assert_eq!(out[0], PenEvent::In { tool: Tool::Pen });
        assert!(matches!(out[1], PenEvent::Pose(p) if p.x == 1.0 && p.pressure == Some(0.5)));
    }

    #[test]
    fn partial_updates_keep_last_values() {
        // The 2026-11-13 evdev/Wine bug: a pressure-only update was read as position (0,0).
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(obs(10.5, 20.25, 0.3), &mut out);
        out.clear();
        t.observe(
            Observation {
                pressure: Some(0.0),
                ..Default::default()
            },
            &mut out,
        );
        match out[0] {
            PenEvent::Pose(p) => {
                assert_eq!((p.x, p.y), (10.5, 20.25));
                assert_eq!(p.pressure, Some(0.0));
            }
            ref e => unreachable_event(e),
        }
    }

    #[test]
    fn no_pose_until_position_known() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(
            Observation {
                pressure: Some(0.2),
                ..Default::default()
            },
            &mut out,
        );
        assert_eq!(out, vec![PenEvent::In { tool: Tool::Pen }]);
    }

    #[test]
    fn up_is_synthesised_before_out() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(obs(0.0, 0.0, 0.5), &mut out);
        t.tip(true, &mut out);
        out.clear();
        t.proximity_out(&mut out);
        assert_eq!(out, vec![PenEvent::Up, PenEvent::Out]);
    }

    #[test]
    fn tool_change_closes_previous_tool() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.proximity_in(Tool::Pen, &mut out);
        t.tip(true, &mut out);
        out.clear();
        t.proximity_in(Tool::Eraser, &mut out);
        assert_eq!(
            out,
            vec![
                PenEvent::Up,
                PenEvent::Out,
                PenEvent::In { tool: Tool::Eraser }
            ]
        );
    }

    #[test]
    fn duplicate_tip_transitions_are_ignored() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.tip(true, &mut out);
        t.tip(true, &mut out);
        t.tip(false, &mut out);
        t.tip(false, &mut out);
        assert_eq!(
            out,
            vec![
                PenEvent::In { tool: Tool::Pen },
                PenEvent::Down,
                PenEvent::Up
            ]
        );
    }

    #[test]
    fn values_are_clamped_and_non_finite_dropped() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        // Give the pen a varying twist first, so twist is reported (see module docs).
        t.observe(
            Observation {
                twist: Some(0.0),
                ..Default::default()
            },
            &mut out,
        );
        t.observe(
            Observation {
                x: Some(1.0),
                y: Some(1.0),
                pressure: Some(2.0),
                tilt_x: Some(-120.0),
                twist: Some(-10.0),
                ..Default::default()
            },
            &mut out,
        );
        t.observe(
            Observation {
                pressure: Some(f32::NAN),
                x: Some(f64::INFINITY),
                ..Default::default()
            },
            &mut out,
        );
        let poses: Vec<PenPose> = out
            .iter()
            .filter_map(|e| {
                if let PenEvent::Pose(p) = e {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(poses[0].pressure, Some(1.0));
        assert_eq!(poses[0].tilt, Some([-90.0, 0.0]));
        assert_eq!(poses[0].twist, Some(350.0));
        assert_eq!((poses[1].x, poses[1].pressure), (1.0, Some(1.0)));
    }

    #[test]
    fn lifting_the_tip_reports_zero_pressure() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(obs(5.0, 5.0, 0.8), &mut out);
        t.tip(true, &mut out);
        t.tip(false, &mut out);
        out.clear();
        t.observe(
            Observation {
                x: Some(6.0),
                ..Default::default()
            },
            &mut out,
        );
        assert!(matches!(out[0], PenEvent::Pose(p) if p.pressure == Some(0.0)));
    }

    fn twists(out: &[PenEvent]) -> Vec<Option<f32>> {
        out.iter()
            .filter_map(|e| {
                if let PenEvent::Pose(p) = e {
                    Some(p.twist)
                } else {
                    None
                }
            })
            .collect()
    }

    fn twist_obs(x: f64, r: f32) -> Observation {
        Observation {
            x: Some(x),
            y: Some(0.0),
            twist: Some(r),
            ..Default::default()
        }
    }

    #[test]
    fn constant_twist_is_not_reported() {
        // OpenTabletDriver + GNOME: a declared but unused rotation axis reads a constant 90°.
        let mut t = PenTracker::new();
        let mut out = vec![];
        for i in 0..5 {
            t.observe(twist_obs(f64::from(i), 90.0), &mut out);
        }
        assert!(twists(&out).iter().all(Option::is_none));
    }

    #[test]
    fn twist_is_reported_once_it_varies_and_stays_reported() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(twist_obs(0.0, 30.0), &mut out);
        t.observe(twist_obs(1.0, 30.0), &mut out);
        t.observe(twist_obs(2.0, 35.0), &mut out);
        t.observe(twist_obs(3.0, 35.0), &mut out);
        assert_eq!(twists(&out), vec![None, None, Some(35.0), Some(35.0)]);
        // Leaving and re-entering proximity keeps it: this tool has real rotation.
        t.proximity_out(&mut out);
        out.clear();
        t.observe(twist_obs(4.0, 35.0), &mut out);
        assert_eq!(twists(&out), vec![Some(35.0)]);
    }

    #[test]
    fn twist_variation_is_tracked_per_tool() {
        let mut t = PenTracker::new();
        let mut out = vec![];
        t.observe(twist_obs(0.0, 10.0), &mut out);
        t.observe(twist_obs(1.0, 20.0), &mut out);
        t.proximity_in(Tool::Eraser, &mut out);
        out.clear();
        t.observe(twist_obs(2.0, 20.0), &mut out);
        assert_eq!(twists(&out), vec![None]);
    }

    fn unreachable_event(e: &PenEvent) -> ! {
        unreachable!("unexpected event {e:?}")
    }
}
