//! The canvas view: pan, zoom and rotation, in physical (device) pixels.
//!
//! screen = zoom · R(angle) · canvas + origin, with `origin` the screen position of the canvas's
//! top-left corner. At 100 % and no rotation the origin is snapped to whole pixels, so every
//! canvas pixel lands on exactly one screen pixel (D-012 4: the display shows the CPU pixels).

/// Zoom limits (fraction of 100 %).
pub const MIN_ZOOM: f64 = 0.01;
pub const MAX_ZOOM: f64 = 64.0;

/// Zoom steps for zoom in/out (CSP-like percentages).
const STEPS: &[f64] = &[
    0.01, 0.02, 0.03, 0.05, 0.0625, 0.0833, 0.125, 0.167, 0.25, 0.333, 0.5, 0.667, 0.75, 1.0, 1.25,
    1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub zoom: f64,
    /// Degrees, clockwise on screen.
    pub angle: f64,
    /// Screen position (physical px) of the canvas origin.
    pub origin: [f64; 2],
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            angle: 0.0,
            origin: [0.0, 0.0],
        }
    }
}

impl View {
    fn rot(&self) -> (f64, f64) {
        let a = self.angle.to_radians();
        (a.sin(), a.cos())
    }

    /// Exact pixel mapping is in effect (100 %, unrotated).
    pub fn is_pixel_exact(&self) -> bool {
        self.zoom == 1.0 && self.angle.rem_euclid(360.0) == 0.0
    }

    /// The 2×2 matrix (row-major) and translation mapping canvas to screen.
    pub fn matrix(&self) -> ([f64; 4], [f64; 2]) {
        let (s, c) = self.rot();
        let z = self.zoom;
        ([z * c, -z * s, z * s, z * c], self.origin)
    }

    pub fn to_screen(&self, p: [f64; 2]) -> [f64; 2] {
        let (m, t) = self.matrix();
        [
            m[0] * p[0] + m[1] * p[1] + t[0],
            m[2] * p[0] + m[3] * p[1] + t[1],
        ]
    }

    pub fn to_canvas(&self, q: [f64; 2]) -> [f64; 2] {
        let (s, c) = self.rot();
        let (dx, dy) = (
            (q[0] - self.origin[0]) / self.zoom,
            (q[1] - self.origin[1]) / self.zoom,
        );
        // Inverse rotation.
        [c * dx + s * dy, -s * dx + c * dy]
    }

    /// Snap the origin to whole pixels when the mapping is 1:1.
    pub fn snap(&mut self) {
        if self.is_pixel_exact() {
            self.origin = [self.origin[0].round(), self.origin[1].round()];
        }
    }

    /// Keep the canvas point under `anchor` (screen px) fixed while changing the view.
    fn keep(&mut self, anchor: [f64; 2], f: impl FnOnce(&mut Self)) {
        let c = self.to_canvas(anchor);
        f(self);
        let moved = self.to_screen(c);
        self.origin[0] += anchor[0] - moved[0];
        self.origin[1] += anchor[1] - moved[1];
        self.snap();
    }

    pub fn set_zoom(&mut self, zoom: f64, anchor: [f64; 2]) {
        let z = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.keep(anchor, |v| v.zoom = z);
    }

    /// Next zoom step up (`dir` > 0) or down.
    pub fn step_zoom(&mut self, dir: i32, anchor: [f64; 2]) {
        let z = self.zoom;
        let next = if dir > 0 {
            STEPS.iter().copied().find(|&s| s > z * 1.0001)
        } else {
            STEPS.iter().rev().copied().find(|&s| s < z / 1.0001)
        };
        if let Some(n) = next {
            self.set_zoom(n, anchor);
        }
    }

    pub fn rotate_by(&mut self, degrees: f64, anchor: [f64; 2]) {
        self.keep(anchor, |v| {
            let a = (v.angle + degrees).rem_euclid(360.0);
            // Land exactly on 0 so the 1:1 mapping comes back.
            v.angle = if a.abs() < 1e-9 || (360.0 - a).abs() < 1e-9 {
                0.0
            } else {
                a
            };
        });
    }

    pub fn pan_by(&mut self, d: [f64; 2]) {
        self.origin[0] += d[0];
        self.origin[1] += d[1];
        self.snap();
    }

    /// Fit a `w`×`h` canvas into the screen rectangle (min, size), unrotated, centred.
    pub fn fit(&mut self, w: u32, h: u32, min: [f64; 2], size: [f64; 2]) {
        let z = ((size[0] * 0.95) / f64::from(w))
            .min((size[1] * 0.95) / f64::from(h))
            .clamp(MIN_ZOOM, 1.0);
        self.zoom = z;
        self.angle = 0.0;
        self.origin = [
            min[0] + (size[0] - f64::from(w) * z) / 2.0,
            min[1] + (size[1] - f64::from(h) * z) / 2.0,
        ];
        self.snap();
    }

    /// Show the canvas at 100 %, centred.
    pub fn actual_size(&mut self, w: u32, h: u32, min: [f64; 2], size: [f64; 2]) {
        self.zoom = 1.0;
        self.angle = 0.0;
        self.origin = [
            min[0] + (size[0] - f64::from(w)) / 2.0,
            min[1] + (size[1] - f64::from(h)) / 2.0,
        ];
        self.snap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
    }

    #[test]
    fn round_trips_and_anchors() {
        let mut v = View::default();
        v.set_zoom(2.5, [100.0, 50.0]);
        v.rotate_by(30.0, [300.0, 200.0]);
        let p = [123.4, 56.7];
        assert!(close(v.to_canvas(v.to_screen(p)), p));
        // The anchor stays put.
        let before = v.to_canvas([400.0, 300.0]);
        v.set_zoom(0.5, [400.0, 300.0]);
        assert!(close(v.to_canvas([400.0, 300.0]), before));
        // Rotating back lands exactly on 1:1 and snaps.
        let mut v = View {
            origin: [10.4, 20.6],
            ..View::default()
        };
        v.rotate_by(90.0, [0.0, 0.0]);
        v.rotate_by(-90.0, [0.0, 0.0]);
        assert!(v.is_pixel_exact());
        assert_eq!(v.origin, [v.origin[0].round(), v.origin[1].round()]);
    }

    #[test]
    fn zoom_steps() {
        let mut v = View::default();
        v.step_zoom(1, [0.0, 0.0]);
        assert_eq!(v.zoom, 1.25);
        v.step_zoom(-1, [0.0, 0.0]);
        v.step_zoom(-1, [0.0, 0.0]);
        assert_eq!(v.zoom, 0.75);
    }
}
