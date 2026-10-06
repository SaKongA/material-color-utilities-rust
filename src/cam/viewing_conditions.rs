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

//! CAM16 viewing conditions and viewing environment parameters.

use crate::utils::{lerp, y_from_lstar, PI, WHITE_POINT_D65};

/// Describes the viewing environment conditions for CAM16 calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewingConditions {
    pub adapting_luminance: f64,
    pub background_lstar: f64,
    pub surround: f64,
    pub discounting_illuminant: bool,
    pub background_y_to_white_point_y: f64,
    pub aw: f64,
    pub nbb: f64,
    pub ncb: f64,
    pub c: f64,
    pub n_c: f64,
    pub fl: f64,
    pub fl_root: f64,
    pub z: f64,
    pub white_point: [f64; 3],
    pub rgb_d: [f64; 3],
}

impl Default for ViewingConditions {
    fn default() -> Self {
        DEFAULT_VIEWING_CONDITIONS
    }
}

/// Creates a `ViewingConditions` structure given viewing environment parameters.
pub fn create_viewing_conditions(
    white_point: [f64; 3],
    adapting_luminance: f64,
    background_lstar: f64,
    surround: f64,
    discounting_illuminant: bool,
) -> ViewingConditions {
    let background_lstar_corrected = if background_lstar < 30.0 {
        30.0
    } else {
        background_lstar
    };

    let rgb_w = [
        0.401288 * white_point[0] + 0.650173 * white_point[1] - 0.051461 * white_point[2],
        -0.250268 * white_point[0] + 1.204414 * white_point[1] + 0.045854 * white_point[2],
        -0.002079 * white_point[0] + 0.048952 * white_point[1] + 0.953127 * white_point[2],
    ];

    let f = 0.8 + (surround / 10.0);
    let c = if f >= 0.9 {
        lerp(0.59, 0.69, (f - 0.9) * 10.0)
    } else {
        lerp(0.525, 0.59, (f - 0.8) * 10.0)
    };

    let mut d = if discounting_illuminant {
        1.0
    } else {
        f * (1.0 - ((1.0 / 3.6) * ((-adapting_luminance - 42.0) / 92.0).exp()))
    };
    d = d.clamp(0.0, 1.0);

    let nc = f;
    let rgb_d = [
        d * (100.0 / rgb_w[0]) + 1.0 - d,
        d * (100.0 / rgb_w[1]) + 1.0 - d,
        d * (100.0 / rgb_w[2]) + 1.0 - d,
    ];

    let k = 1.0 / (5.0 * adapting_luminance + 1.0);
    let k4 = k * k * k * k;
    let k4f = 1.0 - k4;
    let fl =
        (k4 * adapting_luminance) + (0.1 * k4f * k4f * (5.0 * adapting_luminance).powf(1.0 / 3.0));
    let fl_root = fl.powf(0.25);
    let n = y_from_lstar(background_lstar_corrected) / white_point[1];
    let z = 1.48 + n.sqrt();
    let nbb = 0.725 / n.powf(0.2);
    let ncb = nbb;

    let rgb_a_factors = [
        (fl * rgb_d[0] * rgb_w[0] / 100.0).powf(0.42),
        (fl * rgb_d[1] * rgb_w[1] / 100.0).powf(0.42),
        (fl * rgb_d[2] * rgb_w[2] / 100.0).powf(0.42),
    ];

    let rgb_a = [
        400.0 * rgb_a_factors[0] / (rgb_a_factors[0] + 27.13),
        400.0 * rgb_a_factors[1] / (rgb_a_factors[1] + 27.13),
        400.0 * rgb_a_factors[2] / (rgb_a_factors[2] + 27.13),
    ];

    let aw = (40.0 * rgb_a[0] + 20.0 * rgb_a[1] + rgb_a[2]) / 20.0 * nbb;

    ViewingConditions {
        adapting_luminance,
        background_lstar: background_lstar_corrected,
        surround,
        discounting_illuminant,
        background_y_to_white_point_y: n,
        aw,
        nbb,
        ncb,
        c,
        n_c: nc,
        fl,
        fl_root,
        z,
        white_point,
        rgb_d,
    }
}

/// Creates a `ViewingConditions` instance with default values and a custom `background_lstar`.
pub fn default_with_background_lstar(background_lstar: f64) -> ViewingConditions {
    create_viewing_conditions(
        WHITE_POINT_D65,
        200.0 / PI * y_from_lstar(50.0) / 100.0,
        background_lstar,
        2.0,
        false,
    )
}

/// The standard default viewing conditions.
pub const DEFAULT_VIEWING_CONDITIONS: ViewingConditions = ViewingConditions {
    adapting_luminance: 11.725676537,
    background_lstar: 50.000000000,
    surround: 2.000000000,
    discounting_illuminant: false,
    background_y_to_white_point_y: 0.184186503,
    aw: 29.981000900,
    nbb: 1.016919255,
    ncb: 1.016919255,
    c: 0.689999998,
    n_c: 1.000000000,
    fl: 0.388481468,
    fl_root: 0.789482653,
    z: 1.909169555,
    white_point: [95.047, 100.0, 108.883],
    rgb_d: [1.021177769, 0.986307740, 0.933960497],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_default_viewing_conditions_values() {
        let computed = default_with_background_lstar(50.0);
        let expected = DEFAULT_VIEWING_CONDITIONS;

        assert!(double_near(
            computed.adapting_luminance,
            expected.adapting_luminance,
            1e-5
        ));
        assert!(double_near(
            computed.background_lstar,
            expected.background_lstar,
            1e-6
        ));
        assert!(double_near(computed.surround, expected.surround, 1e-6));
        assert_eq!(
            computed.discounting_illuminant,
            expected.discounting_illuminant
        );
        assert!(double_near(
            computed.background_y_to_white_point_y,
            expected.background_y_to_white_point_y,
            1e-6
        ));
        assert!(double_near(computed.aw, expected.aw, 1e-5));
        assert!(double_near(computed.nbb, expected.nbb, 1e-6));
        assert!(double_near(computed.ncb, expected.ncb, 1e-6));
        assert!(double_near(computed.c, expected.c, 1e-6));
        assert!(double_near(computed.n_c, expected.n_c, 1e-6));
        assert!(double_near(computed.fl, expected.fl, 1e-6));
        assert!(double_near(computed.fl_root, expected.fl_root, 1e-6));
        assert!(double_near(computed.z, expected.z, 1e-6));
        assert!(double_near(computed.rgb_d[0], expected.rgb_d[0], 1e-6));
        assert!(double_near(computed.rgb_d[1], expected.rgb_d[1], 1e-6));
        assert!(double_near(computed.rgb_d[2], expected.rgb_d[2], 1e-6));
    }
}
