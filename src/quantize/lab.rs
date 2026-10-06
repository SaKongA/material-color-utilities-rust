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

//! CIE L*a*b* color space representation and conversions.

use crate::utils::{
    argb_from_rgb, blue_from_argb, delinearized, green_from_argb, linearized, red_from_argb, Argb,
    WHITE_POINT_D65,
};

/// CIE 1976 $L^*a^*b^*$ color representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl Lab {
    /// Creates a new `Lab` struct.
    pub const fn new(l: f64, a: f64, b: f64) -> Self {
        Self { l, a, b }
    }

    /// Computes the Euclidean squared color difference $\Delta E^2$ between two Lab colors.
    pub fn delta_e(&self, other: &Lab) -> f64 {
        let d_l = self.l - other.l;
        let d_a = self.a - other.a;
        let d_b = self.b - other.b;
        d_l * d_l + d_a * d_a + d_b * d_b
    }
}

/// Converts a `Lab` color to an ARGB integer.
pub fn int_from_lab(lab: Lab) -> Argb {
    const E: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    const KE: f64 = 8.0;

    let fy = (lab.l + 16.0) / 116.0;
    let fx = (lab.a / 500.0) + fy;
    let fz = fy - (lab.b / 200.0);
    let fx3 = fx * fx * fx;
    let x_normalized = if fx3 > E {
        fx3
    } else {
        (116.0 * fx - 16.0) / KAPPA
    };
    let y_normalized = if lab.l > KE {
        fy * fy * fy
    } else {
        lab.l / KAPPA
    };
    let fz3 = fz * fz * fz;
    let z_normalized = if fz3 > E {
        fz3
    } else {
        (116.0 * fz - 16.0) / KAPPA
    };

    let x = x_normalized * WHITE_POINT_D65[0];
    let y = y_normalized * WHITE_POINT_D65[1];
    let z = z_normalized * WHITE_POINT_D65[2];

    let r_l = 3.2406 * x - 1.5372 * y - 0.4986 * z;
    let g_l = -0.9689 * x + 1.8758 * y + 0.0415 * z;
    let b_l = 0.0557 * x - 0.2040 * y + 1.0570 * z;

    let red = delinearized(r_l);
    let green = delinearized(g_l);
    let blue = delinearized(b_l);

    argb_from_rgb(red, green, blue)
}

/// Converts an ARGB integer to a `Lab` color.
pub fn lab_from_int(argb: Argb) -> Lab {
    let red = red_from_argb(argb) as i32;
    let green = green_from_argb(argb) as i32;
    let blue = blue_from_argb(argb) as i32;
    let red_l = linearized(red);
    let green_l = linearized(green);
    let blue_l = linearized(blue);

    let x = 0.41233895 * red_l + 0.35762064 * green_l + 0.18051042 * blue_l;
    let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
    let z = 0.01932141 * red_l + 0.11916382 * green_l + 0.95034478 * blue_l;

    let y_normalized = y / WHITE_POINT_D65[1];
    const E: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;

    let fy = if y_normalized > E {
        y_normalized.powf(1.0 / 3.0)
    } else {
        (KAPPA * y_normalized + 16.0) / 116.0
    };

    let x_normalized = x / WHITE_POINT_D65[0];
    let fx = if x_normalized > E {
        x_normalized.powf(1.0 / 3.0)
    } else {
        (KAPPA * x_normalized + 16.0) / 116.0
    };

    let z_normalized = z / WHITE_POINT_D65[2];
    let fz = if z_normalized > E {
        z_normalized.powf(1.0 / 3.0)
    } else {
        (KAPPA * z_normalized + 16.0) / 116.0
    };

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    Lab::new(l, a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_lab_roundtrip() {
        let original_argb: Argb = 0xff32_96fa;
        let lab = lab_from_int(original_argb);
        let recovered = int_from_lab(lab);
        assert_eq!(recovered, original_argb);
        assert!(double_near(lab.l, 61.127, 0.01));
    }
}
