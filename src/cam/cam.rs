/*
 * Copyright 2026 SaKongA
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! CAM16 color appearance model implementation.

use crate::cam::hct_solver::solve_to_int;
use crate::cam::viewing_conditions::{ViewingConditions, DEFAULT_VIEWING_CONDITIONS};
use crate::utils::{
    argb_from_rgb, delinearized, linearized, sanitize_degrees_double, signum, Argb, PI,
};

/// CAM16 color appearance model representation of a color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cam {
    pub hue: f64,
    pub chroma: f64,
    pub j: f64,
    pub q: f64,
    pub m: f64,
    pub s: f64,
    pub jstar: f64,
    pub astar: f64,
    pub bstar: f64,
}

impl Cam {
    /// Creates a new `Cam` instance.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        hue: f64,
        chroma: f64,
        j: f64,
        q: f64,
        m: f64,
        s: f64,
        jstar: f64,
        astar: f64,
        bstar: f64,
    ) -> Self {
        Self {
            hue,
            chroma,
            j,
            q,
            m,
            s,
            jstar,
            astar,
            bstar,
        }
    }
}

/// Computes CAM16 parameters from JCH (Lightness J, Chroma C, Hue angle h)
/// under the given viewing conditions.
pub fn cam_from_jch_and_viewing_conditions(
    j: f64,
    c: f64,
    h: f64,
    viewing_conditions: &ViewingConditions,
) -> Cam {
    let q = (4.0 / viewing_conditions.c)
        * (j / 100.0).sqrt()
        * (viewing_conditions.aw + 4.0)
        * viewing_conditions.fl_root;
    let m = c * viewing_conditions.fl_root;
    let alpha = c / (j / 100.0).sqrt();
    let s = 50.0 * ((alpha * viewing_conditions.c) / (viewing_conditions.aw + 4.0)).sqrt();
    let hue_radians = h * PI / 180.0;
    let jstar = (1.0 + 100.0 * 0.007) * j / (1.0 + 0.007 * j);
    let mstar = 1.0 / 0.0228 * (1.0 + 0.0228 * m).ln();
    let astar = mstar * hue_radians.cos();
    let bstar = mstar * hue_radians.sin();

    Cam::new(h, c, j, q, m, s, jstar, astar, bstar)
}

/// Converts CAM16-UCS coordinates ($J^*, a^*, b^*$) to `Cam` under given viewing conditions.
pub fn cam_from_ucs_and_viewing_conditions(
    jstar: f64,
    astar: f64,
    bstar: f64,
    viewing_conditions: &ViewingConditions,
) -> Cam {
    let a = astar;
    let b = bstar;
    let m = (a * a + b * b).sqrt();
    let m_2 = ((m * 0.0228).exp() - 1.0) / 0.0228;
    let c = m_2 / viewing_conditions.fl_root;
    let mut h = b.atan2(a) * (180.0 / PI);
    if h < 0.0 {
        h += 360.0;
    }
    let j = jstar / (1.0 - (jstar - 100.0) * 0.007);
    cam_from_jch_and_viewing_conditions(j, c, h, viewing_conditions)
}

/// Converts CIE XYZ coordinates to `Cam` under given viewing conditions.
pub fn cam_from_xyz_and_viewing_conditions(
    x: f64,
    y: f64,
    z: f64,
    viewing_conditions: &ViewingConditions,
) -> Cam {
    // Convert XYZ to cone/RGB responses
    let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
    let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
    let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;

    // Discount illuminant
    let r_d = viewing_conditions.rgb_d[0] * r_c;
    let g_d = viewing_conditions.rgb_d[1] * g_c;
    let b_d = viewing_conditions.rgb_d[2] * b_c;

    // Chromatic adaptation
    let r_af = (viewing_conditions.fl * r_d.abs() / 100.0).powf(0.42);
    let g_af = (viewing_conditions.fl * g_d.abs() / 100.0).powf(0.42);
    let b_af = (viewing_conditions.fl * b_d.abs() / 100.0).powf(0.42);
    let r_a = (signum(r_d) as f64) * 400.0 * r_af / (r_af + 27.13);
    let g_a = (signum(g_d) as f64) * 400.0 * g_af / (g_af + 27.13);
    let b_a = (signum(b_d) as f64) * 400.0 * b_af / (b_af + 27.13);

    // Redness-greenness and opponent responses
    let a = (11.0 * r_a - 12.0 * g_a + b_a) / 11.0;
    let b = (r_a + g_a - 2.0 * b_a) / 9.0;
    let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
    let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;

    let radians = b.atan2(a);
    let degrees = radians * 180.0 / PI;
    let hue = sanitize_degrees_double(degrees);
    let hue_radians = hue * PI / 180.0;
    let ac = p2 * viewing_conditions.nbb;

    let j = 100.0 * (ac / viewing_conditions.aw).powf(viewing_conditions.c * viewing_conditions.z);
    let q = (4.0 / viewing_conditions.c)
        * (j / 100.0).sqrt()
        * (viewing_conditions.aw + 4.0)
        * viewing_conditions.fl_root;
    let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
    let e_hue = 0.25 * ((hue_prime * PI / 180.0 + 2.0).cos() + 3.8);
    let p1 = 50000.0 / 13.0 * e_hue * viewing_conditions.n_c * viewing_conditions.ncb;
    let t = p1 * (a * a + b * b).sqrt() / (u + 0.305);
    let alpha = t.powf(0.9)
        * (1.64 - 0.29f64.powf(viewing_conditions.background_y_to_white_point_y)).powf(0.73);
    let c = alpha * (j / 100.0).sqrt();
    let m = c * viewing_conditions.fl_root;
    let s = 50.0 * ((alpha * viewing_conditions.c) / (viewing_conditions.aw + 4.0)).sqrt();
    let jstar = (1.0 + 100.0 * 0.007) * j / (1.0 + 0.007 * j);
    let mstar = 1.0 / 0.0228 * (1.0 + 0.0228 * m).ln();
    let astar = mstar * hue_radians.cos();
    let bstar = mstar * hue_radians.sin();

    Cam::new(hue, c, j, q, m, s, jstar, astar, bstar)
}

/// Converts an ARGB color to `Cam` under the given viewing conditions.
pub fn cam_from_int_and_viewing_conditions(
    argb: Argb,
    viewing_conditions: &ViewingConditions,
) -> Cam {
    let red = ((argb & 0x00ff_0000) >> 16) as i32;
    let green = ((argb & 0x0000_ff00) >> 8) as i32;
    let blue = (argb & 0x0000_00ff) as i32;
    let red_l = linearized(red);
    let green_l = linearized(green);
    let blue_l = linearized(blue);
    let x = 0.41233895 * red_l + 0.35762064 * green_l + 0.18051042 * blue_l;
    let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
    let z = 0.01932141 * red_l + 0.11916382 * green_l + 0.95034478 * blue_l;

    cam_from_xyz_and_viewing_conditions(x, y, z, viewing_conditions)
}

/// Converts an ARGB color to `Cam` under default viewing conditions.
pub fn cam_from_int(argb: Argb) -> Cam {
    cam_from_int_and_viewing_conditions(argb, &DEFAULT_VIEWING_CONDITIONS)
}

/// Converts `Cam` to ARGB color under the given viewing conditions.
pub fn int_from_cam_and_viewing_conditions(
    cam: Cam,
    viewing_conditions: &ViewingConditions,
) -> Argb {
    let alpha = if cam.chroma == 0.0 || cam.j == 0.0 {
        0.0
    } else {
        cam.chroma / (cam.j / 100.0).sqrt()
    };
    let t = (alpha
        / (1.64 - 0.29f64.powf(viewing_conditions.background_y_to_white_point_y)).powf(0.73))
    .powf(1.0 / 0.9);
    let h_rad = cam.hue * PI / 180.0;
    let e_hue = 0.25 * ((h_rad + 2.0).cos() + 3.8);
    let ac = viewing_conditions.aw
        * (cam.j / 100.0).powf(1.0 / viewing_conditions.c / viewing_conditions.z);
    let p1 = e_hue * (50000.0 / 13.0) * viewing_conditions.n_c * viewing_conditions.ncb;
    let p2 = ac / viewing_conditions.nbb;
    let h_sin = h_rad.sin();
    let h_cos = h_rad.cos();
    let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * h_cos + 108.0 * t * h_sin);
    let a = gamma * h_cos;
    let b = gamma * h_sin;
    let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
    let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
    let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

    let r_c_base = (27.13 * r_a.abs() / (400.0 - r_a.abs())).max(0.0);
    let r_c = (signum(r_a) as f64) * (100.0 / viewing_conditions.fl) * r_c_base.powf(1.0 / 0.42);
    let g_c_base = (27.13 * g_a.abs() / (400.0 - g_a.abs())).max(0.0);
    let g_c = (signum(g_a) as f64) * (100.0 / viewing_conditions.fl) * g_c_base.powf(1.0 / 0.42);
    let b_c_base = (27.13 * b_a.abs() / (400.0 - b_a.abs())).max(0.0);
    let b_c = (signum(b_a) as f64) * (100.0 / viewing_conditions.fl) * b_c_base.powf(1.0 / 0.42);

    let r_x = r_c / viewing_conditions.rgb_d[0];
    let g_x = g_c / viewing_conditions.rgb_d[1];
    let b_x = b_c / viewing_conditions.rgb_d[2];
    let x = 1.86206786 * r_x - 1.01125463 * g_x + 0.14918677 * b_x;
    let y = 0.38752654 * r_x + 0.62144744 * g_x - 0.00897398 * b_x;
    let z = -0.01584150 * r_x - 0.03412294 * g_x + 1.04996444 * b_x;

    let r_l = 3.2406 * x - 1.5372 * y - 0.4986 * z;
    let g_l = -0.9689 * x + 1.8758 * y + 0.0415 * z;
    let b_l = 0.0557 * x - 0.2040 * y + 1.0570 * z;

    let red = delinearized(r_l);
    let green = delinearized(g_l);
    let blue = delinearized(b_l);

    argb_from_rgb(red, green, blue)
}

/// Converts `Cam` to ARGB color under default viewing conditions.
pub fn int_from_cam(cam: Cam) -> Argb {
    int_from_cam_and_viewing_conditions(cam, &DEFAULT_VIEWING_CONDITIONS)
}

/// Calculates the CAM16-UCS perceptual color distance $\Delta E$ between two `Cam` colors.
pub fn cam_distance(a: Cam, b: Cam) -> f64 {
    let d_j = a.jstar - b.jstar;
    let d_a = a.astar - b.astar;
    let d_b = a.bstar - b.bstar;
    let d_e_prime = (d_j * d_j + d_a * d_a + d_b * d_b).sqrt();
    1.41 * d_e_prime.powf(0.63)
}

/// Converts Hue, Chroma, and Tone (L*) to an ARGB integer.
pub fn int_from_hcl(hue: f64, chroma: f64, lstar: f64) -> Argb {
    solve_to_int(hue, chroma, lstar)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Argb = 0xffff_0000;
    const GREEN: Argb = 0xff00_ff00;
    const BLUE: Argb = 0xff00_00ff;
    const WHITE: Argb = 0xffff_ffff;
    const BLACK: Argb = 0xff00_0000;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_cam_red() {
        let cam = cam_from_int(RED);
        assert!(double_near(cam.hue, 27.408, 0.001));
        assert!(double_near(cam.chroma, 113.357, 0.001));
        assert!(double_near(cam.j, 46.445, 0.001));
        assert!(double_near(cam.m, 89.494, 0.001));
        assert!(double_near(cam.s, 91.889, 0.001));
        assert!(double_near(cam.q, 105.988, 0.001));
    }

    #[test]
    fn test_cam_green() {
        let cam = cam_from_int(GREEN);
        assert!(double_near(cam.hue, 142.139, 0.001));
        assert!(double_near(cam.chroma, 108.410, 0.001));
        assert!(double_near(cam.j, 79.331, 0.001));
        assert!(double_near(cam.m, 85.587, 0.001));
        assert!(double_near(cam.s, 78.604, 0.001));
        assert!(double_near(cam.q, 138.520, 0.001));
    }

    #[test]
    fn test_cam_blue() {
        let cam = cam_from_int(BLUE);
        assert!(double_near(cam.hue, 282.788, 0.001));
        assert!(double_near(cam.chroma, 87.230, 0.001));
        assert!(double_near(cam.j, 25.465, 0.001));
        assert!(double_near(cam.m, 68.867, 0.001));
        assert!(double_near(cam.s, 93.674, 0.001));
        assert!(double_near(cam.q, 78.481, 0.001));
    }

    #[test]
    fn test_cam_white() {
        let cam = cam_from_int(WHITE);
        assert!(double_near(cam.hue, 209.492, 0.001));
        assert!(double_near(cam.chroma, 2.869, 0.001));
        assert!(double_near(cam.j, 100.0, 0.001));
        assert!(double_near(cam.m, 2.265, 0.001));
        assert!(double_near(cam.s, 12.068, 0.001));
        assert!(double_near(cam.q, 155.521, 0.001));
    }

    #[test]
    fn test_cam_black() {
        let cam = cam_from_int(BLACK);
        assert!(double_near(cam.hue, 0.0, 0.001));
        assert!(double_near(cam.chroma, 0.0, 0.001));
        assert!(double_near(cam.j, 0.0, 0.001));
        assert!(double_near(cam.m, 0.0, 0.001));
        assert!(double_near(cam.s, 0.0, 0.001));
        assert!(double_near(cam.q, 0.0, 0.001));
    }

    #[test]
    fn test_cam_red_roundtrip() {
        let cam = cam_from_int(RED);
        let argb = int_from_cam(cam);
        assert_eq!(argb, RED);
    }

    #[test]
    fn test_cam_green_roundtrip() {
        let cam = cam_from_int(GREEN);
        let argb = int_from_cam(cam);
        assert_eq!(argb, GREEN);
    }

    #[test]
    fn test_cam_blue_roundtrip() {
        let cam = cam_from_int(BLUE);
        let argb = int_from_cam(cam);
        assert_eq!(argb, BLUE);
    }
}
