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

//! HCT (Hue, Chroma, Tone) color representation.
//!
//! A perceptually uniform color space combining CAM16 hue and chroma
//! with CIE L* (Tone / lightness).

use std::cmp::Ordering;

use crate::cam::cam::cam_from_int;
use crate::cam::hct_solver::solve_to_int;
use crate::utils::{lstar_from_argb, Argb};

/// HCT: Hue, Chroma, and Tone color space.
///
/// HCT enables intuitive color palette manipulation where tone is
/// linearly correlated with perceived luminance and contrast.
#[derive(Debug, Clone, Copy)]
pub struct Hct {
    hue: f64,
    chroma: f64,
    tone: f64,
    argb: Argb,
}

impl PartialEq for Hct {
    fn eq(&self, other: &Self) -> bool {
        self.argb == other.argb
    }
}

impl Eq for Hct {}

impl PartialOrd for Hct {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hct {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hue.total_cmp(&other.hue)
    }
}

impl Hct {
    /// Creates an HCT color from hue, chroma, and tone under default viewing conditions.
    pub fn new(hue: f64, chroma: f64, tone: f64) -> Self {
        let mut hct = Self {
            hue: 0.0,
            chroma: 0.0,
            tone: 0.0,
            argb: 0,
        };
        hct.set_internal_state(solve_to_int(hue, chroma, tone));
        hct
    }

    /// Creates an HCT color from an ARGB integer.
    pub fn from_int(argb: Argb) -> Self {
        let mut hct = Self {
            hue: 0.0,
            chroma: 0.0,
            tone: 0.0,
            argb: 0,
        };
        hct.set_internal_state(argb);
        hct
    }

    /// Creates an HCT color from an ARGB integer (alias for `from_int`).
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_int(argb)
    }

    /// Returns the hue of the color in degrees `[0.0, 360.0)`.
    #[inline]
    pub fn hue(&self) -> f64 {
        self.hue
    }

    /// Returns the chroma (colorfulness) of the color.
    #[inline]
    pub fn chroma(&self) -> f64 {
        self.chroma
    }

    /// Returns the tone (perceptual lightness L*) of the color `[0.0, 100.0]`.
    #[inline]
    pub fn tone(&self) -> f64 {
        self.tone
    }

    /// Returns the color in 32-bit ARGB format.
    #[inline]
    pub fn to_int(&self) -> Argb {
        self.argb
    }

    /// Sets the hue of this color, recalculating chroma and tone to fit within the sRGB gamut.
    pub fn set_hue(&mut self, new_hue: f64) {
        self.set_internal_state(solve_to_int(new_hue, self.chroma, self.tone));
    }

    /// Sets the chroma of this color, recalculating to fit within the sRGB gamut.
    pub fn set_chroma(&mut self, new_chroma: f64) {
        self.set_internal_state(solve_to_int(self.hue, new_chroma, self.tone));
    }

    /// Sets the tone of this color, recalculating chroma to fit within the sRGB gamut.
    pub fn set_tone(&mut self, new_tone: f64) {
        self.set_internal_state(solve_to_int(self.hue, self.chroma, new_tone));
    }

    fn set_internal_state(&mut self, argb: Argb) {
        self.argb = argb;
        let cam = cam_from_int(argb);
        self.hue = cam.hue;
        self.chroma = cam.chroma;
        self.tone = lstar_from_argb(argb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{blue_from_argb, green_from_argb, red_from_argb};

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    fn is_on_boundary(rgb_component: u8) -> bool {
        rgb_component == 0 || rgb_component == 255
    }

    fn color_is_on_boundary(argb: Argb) -> bool {
        is_on_boundary(red_from_argb(argb))
            || is_on_boundary(green_from_argb(argb))
            || is_on_boundary(blue_from_argb(argb))
    }

    #[test]
    fn test_limited_to_srgb() {
        let hct = Hct::new(120.0, 200.0, 50.0);
        let argb = hct.to_int();

        assert_eq!(cam_from_int(argb).hue, hct.hue());
        assert_eq!(cam_from_int(argb).chroma, hct.chroma());
        assert_eq!(lstar_from_argb(argb), hct.tone());
    }

    #[test]
    fn test_truncates_colors() {
        let mut hct = Hct::new(120.0, 60.0, 50.0);
        let chroma = hct.chroma();
        assert!(chroma < 60.0);

        hct.set_tone(180.0);
        assert!(hct.chroma() < chroma);
    }

    #[test]
    fn test_hct_grid_correctness() {
        let hues = [15, 45, 75, 105, 135, 165, 195, 225, 255, 285, 315, 345];
        let chromas = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let tones = [20, 30, 40, 50, 60, 70, 80];

        for &hue in &hues {
            for &chroma in &chromas {
                for &tone in &tones {
                    let color = Hct::new(hue as f64, chroma as f64, tone as f64);

                    if chroma > 0 {
                        assert!(
                            double_near(color.hue(), hue as f64, 4.0),
                            "hue failed for H:{hue} C:{chroma} T:{tone}, got {}",
                            color.hue()
                        );
                    }

                    assert!(
                        color.chroma() < (chroma as f64) + 2.5,
                        "chroma upper bound failed for H:{hue} C:{chroma} T:{tone}"
                    );

                    if color.chroma() < (chroma as f64) - 2.5 {
                        assert!(
                            color_is_on_boundary(color.to_int()),
                            "boundary test failed for H:{hue} C:{chroma} T:{tone}"
                        );
                    }

                    assert!(
                        double_near(color.tone(), tone as f64, 0.5),
                        "tone failed for H:{hue} C:{chroma} T:{tone}, got {}",
                        color.tone()
                    );
                }
            }
        }
    }
}
