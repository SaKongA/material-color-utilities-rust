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

//! Color blend and harmonization utilities.

use crate::cam::cam::{
    cam_from_int, cam_from_ucs_and_viewing_conditions, int_from_cam, Cam,
};
use crate::cam::hct::Hct;
use crate::cam::viewing_conditions::DEFAULT_VIEWING_CONDITIONS;
use crate::utils::{diff_degrees, rotation_direction, sanitize_degrees_double, Argb};

/// Harmonizes a design color with a key/theme color by slightly shifting its hue.
pub fn blend_harmonize(design_color: Argb, key_color: Argb) -> Argb {
    let mut from_hct = Hct::from_int(design_color);
    let to_hct = Hct::from_int(key_color);
    let difference_degrees = diff_degrees(from_hct.hue(), to_hct.hue());
    let rotation_degrees = (difference_degrees * 0.5).min(15.0);
    let output_hue = sanitize_degrees_double(
        from_hct.hue()
            + rotation_degrees * rotation_direction(from_hct.hue(), to_hct.hue()),
    );
    from_hct.set_hue(output_hue);
    from_hct.to_int()
}

/// Blends two colors using CAM16-UCS and transfers the resulting hue onto the `from` color.
pub fn blend_hct_hue(from: Argb, to: Argb, amount: f64) -> Argb {
    let ucs = blend_cam16_ucs(from, to, amount);
    let ucs_hct = Hct::from_int(ucs);
    let mut from_hct = Hct::from_int(from);
    from_hct.set_hue(ucs_hct.hue());
    from_hct.to_int()
}

/// Blends two colors in CAM16 Uniform Color Space (UCS) by linearly interpolating $J^*, a^*, b^*$.
pub fn blend_cam16_ucs(from: Argb, to: Argb, amount: f64) -> Argb {
    let from_cam: Cam = cam_from_int(from);
    let to_cam: Cam = cam_from_int(to);

    let a_j = from_cam.jstar;
    let a_a = from_cam.astar;
    let a_b = from_cam.bstar;

    let b_j = to_cam.jstar;
    let b_a = to_cam.astar;
    let b_b = to_cam.bstar;

    let jstar = a_j + (b_j - a_j) * amount;
    let astar = a_a + (b_a - a_a) * amount;
    let bstar = a_b + (b_b - a_b) * amount;

    let blended = cam_from_ucs_and_viewing_conditions(
        jstar,
        astar,
        bstar,
        &DEFAULT_VIEWING_CONDITIONS,
    );
    int_from_cam(blended)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hex_from_argb;

    #[test]
    fn test_blend_red_to_blue() {
        let blended = blend_hct_hue(0xffff_0000, 0xff00_00ff, 0.8);
        assert_eq!(hex_from_argb(blended), "ff905eff");
    }
}
