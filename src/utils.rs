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

//! Utility types and functions for color space conversion, mathematics,
//! and component manipulation.

/// ARGB color format represented as a 32-bit unsigned integer.
pub type Argb = u32;

/// A 3-dimensional row vector of `f64`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Vec3 {
    /// Creates a new `Vec3`.
    pub const fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }
}

/// The mathematical constant $\pi$.
pub const PI: f64 = std::f64::consts::PI;

/// The standard D65 illuminant white point in CIE 1931 XYZ coordinates ($Y=100$).
pub const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// Returns the red channel of an ARGB integer (0-255).
#[inline]
pub fn red_from_argb(argb: Argb) -> u8 {
    ((argb & 0x00ff_0000) >> 16) as u8
}

/// Returns the green channel of an ARGB integer (0-255).
#[inline]
pub fn green_from_argb(argb: Argb) -> u8 {
    ((argb & 0x0000_ff00) >> 8) as u8
}

/// Returns the blue channel of an ARGB integer (0-255).
#[inline]
pub fn blue_from_argb(argb: Argb) -> u8 {
    (argb & 0x0000_00ff) as u8
}

/// Returns the alpha channel of an ARGB integer (0-255).
#[inline]
pub fn alpha_from_argb(argb: Argb) -> u8 {
    ((argb & 0xff00_0000) >> 24) as u8
}

/// Converts RGB components into an opaque ARGB color (`0xFF_RR_GG_BB`).
#[inline]
pub fn argb_from_rgb(red: i32, green: i32, blue: i32) -> Argb {
    0xff00_0000
        | (((red as u32) & 0xff) << 16)
        | (((green as u32) & 0xff) << 8)
        | ((blue as u32) & 0xff)
}

/// Converts RGBA components into an ARGB integer (`0xAARRGGBB`).
#[inline]
pub fn argb_from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Argb {
    ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

/// Returns the (red, green, blue, alpha) components of an ARGB integer.
#[inline]
pub fn rgba_from_argb(argb: Argb) -> (u8, u8, u8, u8) {
    (
        red_from_argb(argb),
        green_from_argb(argb),
        blue_from_argb(argb),
        alpha_from_argb(argb),
    )
}

/// Linearizes an 8-bit sRGB color component in the range `[0, 255]`
/// to linear RGB in the range `[0.0, 100.0]`.
pub fn linearized(rgb_component: i32) -> f64 {
    let normalized = (rgb_component as f64) / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.0
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4) * 100.0
    }
}

/// Delinearizes a linear color component in the range `[0.0, 100.0]`
/// back to standard 8-bit sRGB in `[0, 255]`.
pub fn delinearized(rgb_component: f64) -> i32 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1.0 / 2.4) - 0.055
    };
    (delinearized * 255.0).round().clamp(0.0, 255.0) as i32
}

/// Converts linear RGB components to an opaque ARGB integer.
pub fn argb_from_linrgb(linrgb: Vec3) -> Argb {
    let r = delinearized(linrgb.a);
    let g = delinearized(linrgb.b);
    let b = delinearized(linrgb.c);
    argb_from_rgb(r, g, b)
}

/// Returns whether an ARGB color has full opacity (`alpha == 255`).
#[inline]
pub fn is_opaque(argb: Argb) -> bool {
    alpha_from_argb(argb) == 255
}

/// Converts CIE L* (in `[0.0, 100.0]`) to relative luminance CIE Y (in `[0.0, 100.0]`).
pub fn y_from_lstar(lstar: f64) -> f64 {
    const KE: f64 = 8.0;
    if lstar > KE {
        let cube_root = (lstar + 16.0) / 116.0;
        let cube = cube_root * cube_root * cube_root;
        cube * 100.0
    } else {
        lstar / (24389.0 / 27.0) * 100.0
    }
}

/// Converts relative luminance CIE Y (in `[0.0, 100.0]`) to CIE L* (in `[0.0, 100.0]`).
pub fn lstar_from_y(y: f64) -> f64 {
    const E: f64 = 216.0 / 24389.0;
    let y_normalized = y / 100.0;
    if y_normalized <= E {
        (24389.0 / 27.0) * y_normalized
    } else {
        116.0 * y_normalized.powf(1.0 / 3.0) - 16.0
    }
}

/// Computes the CIE L* perceptual lightness of an ARGB color.
pub fn lstar_from_argb(argb: Argb) -> f64 {
    let red = red_from_argb(argb) as i32;
    let green = green_from_argb(argb) as i32;
    let blue = blue_from_argb(argb) as i32;
    let red_l = linearized(red);
    let green_l = linearized(green);
    let blue_l = linearized(blue);
    let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
    lstar_from_y(y)
}

/// Sanitizes an angle in degrees as an integer into `[0, 360)`.
#[inline]
pub fn sanitize_degrees_int(degrees: i32) -> i32 {
    if degrees < 0 {
        (degrees % 360) + 360
    } else if degrees >= 360 {
        degrees % 360
    } else {
        degrees
    }
}

/// Sanitizes an angle in degrees as a floating-point number into `[0.0, 360.0)`.
#[inline]
pub fn sanitize_degrees_double(degrees: f64) -> f64 {
    if degrees < 0.0 {
        (degrees % 360.0) + 360.0
    } else if degrees >= 360.0 {
        degrees % 360.0
    } else {
        degrees
    }
}

/// Computes the shortest distance between two angles on a circle in degrees `[0.0, 180.0]`.
#[inline]
pub fn diff_degrees(a: f64, b: f64) -> f64 {
    180.0 - ((a - b).abs() - 180.0).abs()
}

/// Returns the sign of the direction change required to travel from angle `from` to angle `to`.
///
/// Returns `1.0` if increasing `from` is the shortest path (or if the angles are 180° apart),
/// and `-1.0` if decreasing `from` is shortest.
pub fn rotation_direction(from: f64, to: f64) -> f64 {
    let increasing_difference = sanitize_degrees_double(to - from);
    if increasing_difference <= 180.0 {
        1.0
    } else {
        -1.0
    }
}

/// Converts an ARGB color to an 8-character lowercase hexadecimal string (e.g. `"ff89bce1"`).
pub fn hex_from_argb(argb: Argb) -> String {
    format!("{:08x}", argb)
}

/// Returns an ARGB grayscale color with perceptual lightness matching `lstar`.
pub fn int_from_lstar(lstar: f64) -> Argb {
    let y = y_from_lstar(lstar);
    let component = delinearized(y);
    argb_from_rgb(component, component, component)
}

/// Returns the signum of a number: `1` if `num > 0`, `-1` if `num < 0`, and `0` if `num == 0`.
#[inline]
pub fn signum(num: f64) -> i32 {
    if num < 0.0 {
        -1
    } else if num == 0.0 {
        0
    } else {
        1
    }
}

/// Performs linear interpolation between `start` and `stop` with `amount`.
#[inline]
pub fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
    (1.0 - amount) * start + amount * stop
}

/// Multiplies a 1x3 row vector with a 3x3 matrix, returning the resulting 1x3 row vector.
pub fn matrix_multiply(input: Vec3, matrix: &[[f64; 3]; 3]) -> Vec3 {
    let a = input.a * matrix[0][0] + input.b * matrix[0][1] + input.c * matrix[0][2];
    let b = input.a * matrix[1][0] + input.b * matrix[1][1] + input.c * matrix[1][2];
    let c = input.a * matrix[2][0] + input.b * matrix[2][1] + input.c * matrix[2][2];
    Vec3::new(a, b, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATRIX: [[f64; 3]; 3] = [[1.0, 2.0, 3.0], [-4.0, 5.0, -6.0], [-7.0, -8.0, -9.0]];

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn argb_from_rgb_black() {
        assert_eq!(argb_from_rgb(0, 0, 0), 0xff00_0000);
        assert_eq!(argb_from_rgb(0, 0, 0), 4278190080);
    }

    #[test]
    fn argb_from_rgb_white() {
        assert_eq!(argb_from_rgb(255, 255, 255), 0xffff_ffff);
        assert_eq!(argb_from_rgb(255, 255, 255), 4294967295);
    }

    #[test]
    fn argb_from_rgb_random_color() {
        assert_eq!(argb_from_rgb(50, 150, 250), 0xff32_96fa);
        assert_eq!(argb_from_rgb(50, 150, 250), 4281505530);
    }

    #[test]
    fn signum_tests() {
        assert_eq!(signum(0.001), 1);
        assert_eq!(signum(3.0), 1);
        assert_eq!(signum(100.0), 1);
        assert_eq!(signum(-0.002), -1);
        assert_eq!(signum(-4.0), -1);
        assert_eq!(signum(-101.0), -1);
        assert_eq!(signum(0.0), 0);
    }

    #[test]
    fn rotation_is_positive_for_counterclockwise() {
        assert_eq!(rotation_direction(0.0, 30.0), 1.0);
        assert_eq!(rotation_direction(0.0, 60.0), 1.0);
        assert_eq!(rotation_direction(0.0, 150.0), 1.0);
        assert_eq!(rotation_direction(90.0, 240.0), 1.0);
        assert_eq!(rotation_direction(300.0, 30.0), 1.0);
        assert_eq!(rotation_direction(270.0, 60.0), 1.0);
        assert_eq!(rotation_direction(360.0 * 2.0, 15.0), 1.0);
        assert_eq!(
            rotation_direction(360.0 * 3.0 + 15.0, -360.0 * 4.0 + 30.0),
            1.0
        );
    }

    #[test]
    fn rotation_is_negative_for_clockwise() {
        assert_eq!(rotation_direction(30.0, 0.0), -1.0);
        assert_eq!(rotation_direction(60.0, 0.0), -1.0);
        assert_eq!(rotation_direction(150.0, 0.0), -1.0);
        assert_eq!(rotation_direction(240.0, 90.0), -1.0);
        assert_eq!(rotation_direction(30.0, 300.0), -1.0);
        assert_eq!(rotation_direction(60.0, 270.0), -1.0);
        assert_eq!(rotation_direction(15.0, -360.0 * 2.0), -1.0);
        assert_eq!(
            rotation_direction(-360.0 * 4.0 + 270.0, 360.0 * 5.0 + 180.0),
            -1.0
        );
    }

    #[test]
    fn angle_difference() {
        assert_eq!(diff_degrees(0.0, 30.0), 30.0);
        assert_eq!(diff_degrees(0.0, 60.0), 60.0);
        assert_eq!(diff_degrees(0.0, 150.0), 150.0);
        assert_eq!(diff_degrees(90.0, 240.0), 150.0);
        assert_eq!(diff_degrees(300.0, 30.0), 90.0);
        assert_eq!(diff_degrees(270.0, 60.0), 150.0);

        assert_eq!(diff_degrees(30.0, 0.0), 30.0);
        assert_eq!(diff_degrees(60.0, 0.0), 60.0);
        assert_eq!(diff_degrees(150.0, 0.0), 150.0);
        assert_eq!(diff_degrees(240.0, 90.0), 150.0);
        assert_eq!(diff_degrees(30.0, 300.0), 90.0);
        assert_eq!(diff_degrees(60.0, 270.0), 150.0);
    }

    #[test]
    fn angle_sanitation() {
        assert_eq!(sanitize_degrees_int(30), 30);
        assert_eq!(sanitize_degrees_int(240), 240);
        assert_eq!(sanitize_degrees_int(360), 0);
        assert_eq!(sanitize_degrees_int(-30), 330);
        assert_eq!(sanitize_degrees_int(-750), 330);
        assert_eq!(sanitize_degrees_int(-54321), 39);

        assert!(double_near(sanitize_degrees_double(30.0), 30.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(240.0), 240.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(360.0), 0.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(-30.0), 330.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(-750.0), 330.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(-54321.0), 39.0, 1e-4));
        assert!(double_near(sanitize_degrees_double(360.125), 0.125, 1e-4));
        assert!(double_near(sanitize_degrees_double(-11111.11), 48.89, 1e-4));
    }

    #[test]
    fn test_matrix_multiply() {
        let vector_one = matrix_multiply(Vec3::new(1.0, 3.0, 5.0), &MATRIX);
        assert!(double_near(vector_one.a, 22.0, 1e-4));
        assert!(double_near(vector_one.b, -19.0, 1e-4));
        assert!(double_near(vector_one.c, -76.0, 1e-4));

        let vector_two = matrix_multiply(Vec3::new(-11.1, 22.2, -33.3), &MATRIX);
        assert!(double_near(vector_two.a, -66.6, 1e-4));
        assert!(double_near(vector_two.b, 355.2, 1e-4));
        assert!(double_near(vector_two.c, 199.8, 1e-4));
    }

    #[test]
    fn channel_extractions() {
        assert_eq!(alpha_from_argb(0xff12_3456), 0xff);
        assert_eq!(alpha_from_argb(0xffab_cdef), 0xff);

        assert_eq!(red_from_argb(0xff12_3456), 0x12);
        assert_eq!(red_from_argb(0xffab_cdef), 0xab);

        assert_eq!(green_from_argb(0xff12_3456), 0x34);
        assert_eq!(green_from_argb(0xffab_cdef), 0xcd);

        assert_eq!(blue_from_argb(0xff12_3456), 0x56);
        assert_eq!(blue_from_argb(0xffab_cdef), 0xef);

        assert!(is_opaque(0xff12_3456));
        assert!(!is_opaque(0xf012_3456));
        assert!(!is_opaque(0x0012_3456));
    }

    #[test]
    fn linearized_components() {
        assert!(double_near(linearized(0), 0.0, 1e-4));
        assert!(double_near(linearized(1), 0.0303527, 1e-4));
        assert!(double_near(linearized(2), 0.0607054, 1e-4));
        assert!(double_near(linearized(8), 0.242822, 1e-4));
        assert!(double_near(linearized(9), 0.273174, 1e-4));
        assert!(double_near(linearized(16), 0.518152, 1e-4));
        assert!(double_near(linearized(32), 1.44438, 1e-4));
        assert!(double_near(linearized(64), 5.12695, 1e-4));
        assert!(double_near(linearized(128), 21.5861, 1e-4));
        assert!(double_near(linearized(255), 100.0, 1e-4));
    }

    #[test]
    fn delinearized_components() {
        assert_eq!(delinearized(0.0), 0);
        assert_eq!(delinearized(0.0303527), 1);
        assert_eq!(delinearized(0.0607054), 2);
        assert_eq!(delinearized(0.242822), 8);
        assert_eq!(delinearized(0.273174), 9);
        assert_eq!(delinearized(0.518152), 16);
        assert_eq!(delinearized(1.44438), 32);
        assert_eq!(delinearized(5.12695), 64);
        assert_eq!(delinearized(21.5861), 128);
        assert_eq!(delinearized(100.0), 255);

        assert_eq!(delinearized(25.0), 137);
        assert_eq!(delinearized(50.0), 188);
        assert_eq!(delinearized(75.0), 225);

        // Clamping behavior
        assert_eq!(delinearized(-1.0), 0);
        assert_eq!(delinearized(-10000.0), 0);
        assert_eq!(delinearized(101.0), 255);
        assert_eq!(delinearized(10000.0), 255);
    }

    #[test]
    fn delinearized_is_left_inverse_of_linearized() {
        for c in [0, 1, 2, 8, 9, 16, 32, 64, 128, 255] {
            assert_eq!(delinearized(linearized(c)), c);
        }
    }

    #[test]
    fn test_argb_from_linrgb() {
        assert_eq!(argb_from_linrgb(Vec3::new(25.0, 50.0, 75.0)), 0xff89_bce1);
        assert_eq!(argb_from_linrgb(Vec3::new(0.03, 0.06, 0.12)), 0xff01_0204);
    }

    #[test]
    fn test_lstar_from_argb() {
        assert!(double_near(lstar_from_argb(0xff89_bce1), 74.011, 1e-3));
        assert!(double_near(lstar_from_argb(0xff01_0204), 0.529651, 1e-4));
    }

    #[test]
    fn test_hex_from_argb() {
        assert_eq!(hex_from_argb(0xff89_bce1), "ff89bce1");
        assert_eq!(hex_from_argb(0xff01_0204), "ff010204");
    }

    #[test]
    fn test_int_from_lstar() {
        assert_eq!(int_from_lstar(0.0), 0xff00_0000);
        assert_eq!(int_from_lstar(0.25), 0xff01_0101);
        assert_eq!(int_from_lstar(0.5), 0xff02_0202);
        assert_eq!(int_from_lstar(1.0), 0xff04_0404);
        assert_eq!(int_from_lstar(2.0), 0xff07_0707);
        assert_eq!(int_from_lstar(4.0), 0xff0e_0e0e);
        assert_eq!(int_from_lstar(8.0), 0xff18_1818);
        assert_eq!(int_from_lstar(25.0), 0xff3b_3b3b);
        assert_eq!(int_from_lstar(50.0), 0xff77_7777);
        assert_eq!(int_from_lstar(75.0), 0xffb9_b9b9);
        assert_eq!(int_from_lstar(99.0), 0xfffc_fcfc);
        assert_eq!(int_from_lstar(100.0), 0xffff_ffff);

        assert_eq!(int_from_lstar(-1.0), 0xff00_0000);
        assert_eq!(int_from_lstar(-2.0), 0xff00_0000);
        assert_eq!(int_from_lstar(-3.0), 0xff00_0000);
        assert_eq!(int_from_lstar(-9999999.0), 0xff00_0000);

        assert_eq!(int_from_lstar(101.0), 0xffff_ffff);
        assert_eq!(int_from_lstar(111.0), 0xffff_ffff);
        assert_eq!(int_from_lstar(9999999.0), 0xffff_ffff);
    }

    #[test]
    fn test_lstar_argb_roundtrip() {
        for &lstar in &[0.0, 1.0, 2.0, 8.0, 25.0, 50.0, 75.0, 99.0, 100.0] {
            assert!(double_near(
                lstar_from_argb(int_from_lstar(lstar)),
                lstar,
                1.0
            ));
        }
    }

    #[test]
    fn test_argb_lstar_roundtrip() {
        for &color in &[
            0xff00_0000,
            0xff01_0101,
            0xff02_0202,
            0xff11_1111,
            0xff33_3333,
            0xff77_7777,
            0xffbb_bbbb,
            0xfffe_fefe,
            0xffff_ffff,
        ] {
            assert_eq!(int_from_lstar(lstar_from_argb(color)), color);
        }
    }

    #[test]
    fn test_y_from_lstar() {
        assert!(double_near(y_from_lstar(0.0), 0.0, 1e-5));
        assert!(double_near(y_from_lstar(0.1), 0.0110705, 1e-5));
        assert!(double_near(y_from_lstar(0.2), 0.0221411, 1e-5));
        assert!(double_near(y_from_lstar(0.3), 0.0332116, 1e-5));
        assert!(double_near(y_from_lstar(0.4), 0.0442822, 1e-5));
        assert!(double_near(y_from_lstar(0.5), 0.0553528, 1e-5));
        assert!(double_near(y_from_lstar(1.0), 0.1107056, 1e-5));
        assert!(double_near(y_from_lstar(2.0), 0.2214112, 1e-5));
        assert!(double_near(y_from_lstar(3.0), 0.3321169, 1e-5));
        assert!(double_near(y_from_lstar(4.0), 0.4428225, 1e-5));
        assert!(double_near(y_from_lstar(5.0), 0.5535282, 1e-5));
        assert!(double_near(y_from_lstar(8.0), 0.8856451, 1e-5));
        assert!(double_near(y_from_lstar(10.0), 1.1260199, 1e-5));
        assert!(double_near(y_from_lstar(15.0), 1.9085832, 1e-5));
        assert!(double_near(y_from_lstar(20.0), 2.9890524, 1e-5));
        assert!(double_near(y_from_lstar(25.0), 4.4154767, 1e-5));
        assert!(double_near(y_from_lstar(30.0), 6.2359055, 1e-5));
        assert!(double_near(y_from_lstar(40.0), 11.2509737, 1e-5));
        assert!(double_near(y_from_lstar(50.0), 18.4186518, 1e-5));
        assert!(double_near(y_from_lstar(60.0), 28.1233342, 1e-5));
        assert!(double_near(y_from_lstar(70.0), 40.7494157, 1e-5));
        assert!(double_near(y_from_lstar(80.0), 56.6812907, 1e-5));
        assert!(double_near(y_from_lstar(90.0), 76.3033539, 1e-5));
        assert!(double_near(y_from_lstar(95.0), 87.6183294, 1e-5));
        assert!(double_near(y_from_lstar(99.0), 97.4360239, 1e-5));
        assert!(double_near(y_from_lstar(100.0), 100.0, 1e-5));
    }

    #[test]
    fn test_lstar_from_y() {
        assert!(double_near(lstar_from_y(0.0), 0.0, 1e-5));
        assert!(double_near(lstar_from_y(0.1), 0.9032962, 1e-5));
        assert!(double_near(lstar_from_y(0.2), 1.8065925, 1e-5));
        assert!(double_near(lstar_from_y(0.3), 2.7098888, 1e-5));
        assert!(double_near(lstar_from_y(0.4), 3.6131851, 1e-5));
        assert!(double_near(lstar_from_y(0.5), 4.5164814, 1e-5));
        assert!(double_near(lstar_from_y(0.8856451), 8.0, 1e-5));
        assert!(double_near(lstar_from_y(1.0), 8.9914424, 1e-5));
        assert!(double_near(lstar_from_y(2.0), 15.4872443, 1e-5));
        assert!(double_near(lstar_from_y(3.0), 20.0438970, 1e-5));
        assert!(double_near(lstar_from_y(4.0), 23.6714419, 1e-5));
        assert!(double_near(lstar_from_y(5.0), 26.7347653, 1e-5));
        assert!(double_near(lstar_from_y(10.0), 37.8424304, 1e-5));
        assert!(double_near(lstar_from_y(15.0), 45.6341970, 1e-5));
        assert!(double_near(lstar_from_y(20.0), 51.8372115, 1e-5));
        assert!(double_near(lstar_from_y(25.0), 57.0754208, 1e-5));
        assert!(double_near(lstar_from_y(30.0), 61.6542222, 1e-5));
        assert!(double_near(lstar_from_y(40.0), 69.4695307, 1e-5));
        assert!(double_near(lstar_from_y(50.0), 76.0692610, 1e-5));
        assert!(double_near(lstar_from_y(60.0), 81.8381891, 1e-5));
        assert!(double_near(lstar_from_y(70.0), 86.9968642, 1e-5));
        assert!(double_near(lstar_from_y(80.0), 91.6848609, 1e-5));
        assert!(double_near(lstar_from_y(90.0), 95.9967686, 1e-5));
        assert!(double_near(lstar_from_y(95.0), 98.0335184, 1e-5));
        assert!(double_near(lstar_from_y(99.0), 99.6120372, 1e-5));
        assert!(double_near(lstar_from_y(100.0), 100.0, 1e-5));
    }

    #[test]
    fn test_y_lstar_roundtrip() {
        let mut y = 0.0;
        while y <= 100.0 {
            let lstar = lstar_from_y(y);
            let reconstructed = y_from_lstar(lstar);
            assert!(double_near(reconstructed, y, 1e-8));
            y += 0.1;
        }
    }

    #[test]
    fn test_lstar_y_roundtrip() {
        let mut lstar = 0.0;
        while lstar <= 100.0 {
            let y = y_from_lstar(lstar);
            let reconstructed = lstar_from_y(y);
            assert!(double_near(reconstructed, lstar, 1e-8));
            lstar += 0.1;
        }
    }
}
