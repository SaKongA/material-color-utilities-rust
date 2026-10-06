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

//! Tonal palettes and key color calculation.

use std::collections::HashMap;

use crate::cam::cam::cam_from_int;
use crate::cam::hct::Hct;
use crate::cam::hct_solver::solve_to_int;
use crate::utils::Argb;

/// Key color is a color that represents the hue and chroma of a tonal palette.
#[derive(Debug, Clone)]
pub struct KeyColor {
    hue: f64,
    requested_chroma: f64,
    max_chroma_value: f64,
    chroma_cache: HashMap<i32, f64>,
}

impl KeyColor {
    /// Creates a new `KeyColor` finder for the given hue and requested chroma.
    pub fn new(hue: f64, requested_chroma: f64) -> Self {
        Self {
            hue,
            requested_chroma,
            max_chroma_value: 200.0,
            chroma_cache: HashMap::new(),
        }
    }

    /// Creates a key color from a hue and requested chroma.
    ///
    /// The key color is the first tone, searching outward from Tone 50,
    /// matching the given hue and chroma.
    pub fn create(&mut self) -> Hct {
        const PIVOT_TONE: i32 = 50;
        const TONE_STEP_SIZE: i32 = 1;
        const EPSILON: f64 = 0.01;

        let mut lower_tone = 0;
        let mut upper_tone = 100;

        while lower_tone < upper_tone {
            let mid_tone = (lower_tone + upper_tone) / 2;
            let is_ascending = self.max_chroma(mid_tone) < self.max_chroma(mid_tone + TONE_STEP_SIZE);
            let sufficient_chroma = self.max_chroma(mid_tone) >= self.requested_chroma - EPSILON;

            if sufficient_chroma {
                if (lower_tone - PIVOT_TONE).abs() < (upper_tone - PIVOT_TONE).abs() {
                    upper_tone = mid_tone;
                } else {
                    if lower_tone == mid_tone {
                        return Hct::new(self.hue, self.requested_chroma, lower_tone as f64);
                    }
                    lower_tone = mid_tone;
                }
            } else if is_ascending {
                lower_tone = mid_tone + TONE_STEP_SIZE;
            } else {
                upper_tone = mid_tone;
            }
        }

        Hct::new(self.hue, self.requested_chroma, lower_tone as f64)
    }

    fn max_chroma(&mut self, tone: i32) -> f64 {
        if let Some(&chroma) = self.chroma_cache.get(&tone) {
            return chroma;
        }
        let chroma = Hct::new(self.hue, self.max_chroma_value, tone as f64).chroma();
        self.chroma_cache.insert(tone, chroma);
        chroma
    }
}

/// A Tonal Palette provides shades of a color with different lightness (Tone) values.
#[derive(Debug, Clone, PartialEq)]
pub struct TonalPalette {
    hue: f64,
    chroma: f64,
    key_color: Hct,
}

impl TonalPalette {
    /// Creates a tonal palette from an ARGB integer.
    pub fn from_argb(argb: Argb) -> Self {
        let cam = cam_from_int(argb);
        let mut key_color_finder = KeyColor::new(cam.hue, cam.chroma);
        let key_color = key_color_finder.create();
        Self {
            hue: cam.hue,
            chroma: cam.chroma,
            key_color,
        }
    }

    /// Creates a tonal palette from an HCT color.
    pub fn from_hct(hct: Hct) -> Self {
        Self {
            hue: hct.hue(),
            chroma: hct.chroma(),
            key_color: hct,
        }
    }

    /// Creates a tonal palette from hue and chroma.
    pub fn from_hue_and_chroma(hue: f64, chroma: f64) -> Self {
        let mut key_color_finder = KeyColor::new(hue, chroma);
        let key_color = key_color_finder.create();
        Self {
            hue,
            chroma,
            key_color,
        }
    }

    /// Creates a tonal palette from hue, chroma, and an explicit key color.
    pub fn from_hue_and_chroma_with_key_color(hue: f64, chroma: f64, key_color: Hct) -> Self {
        Self {
            hue,
            chroma,
            key_color,
        }
    }

    /// Returns the ARGB color for a given tone (lightness) in this palette.
    pub fn get(&self, tone: f64) -> Argb {
        solve_to_int(self.hue, self.chroma, tone)
    }

    /// Returns the hue of the palette.
    #[inline]
    pub fn hue(&self) -> f64 {
        self.hue
    }

    /// Returns the chroma of the palette.
    #[inline]
    pub fn chroma(&self) -> f64 {
        self.chroma
    }

    /// Returns the key color of the palette.
    #[inline]
    pub fn key_color(&self) -> Hct {
        self.key_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hex_from_argb;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_tonal_palette_blue() {
        let color: Argb = 0xff00_00ff;
        let tonal_palette = TonalPalette::from_argb(color);
        assert_eq!(hex_from_argb(tonal_palette.get(100.0)), "ffffffff");
        assert_eq!(hex_from_argb(tonal_palette.get(95.0)), "fff1efff");
        assert_eq!(hex_from_argb(tonal_palette.get(90.0)), "ffe0e0ff");
        assert_eq!(hex_from_argb(tonal_palette.get(80.0)), "ffbec2ff");
        assert_eq!(hex_from_argb(tonal_palette.get(70.0)), "ff9da3ff");
        assert_eq!(hex_from_argb(tonal_palette.get(60.0)), "ff7c84ff");
        assert_eq!(hex_from_argb(tonal_palette.get(50.0)), "ff5a64ff");
        assert_eq!(hex_from_argb(tonal_palette.get(40.0)), "ff343dff");
        assert_eq!(hex_from_argb(tonal_palette.get(30.0)), "ff0000ef");
        assert_eq!(hex_from_argb(tonal_palette.get(20.0)), "ff0001ac");
        assert_eq!(hex_from_argb(tonal_palette.get(10.0)), "ff00006e");
        assert_eq!(hex_from_argb(tonal_palette.get(0.0)), "ff000000");
    }

    #[test]
    fn test_key_color_exact_chroma_available() {
        let palette = TonalPalette::from_hue_and_chroma(50.0, 60.0);
        let result = palette.key_color();

        assert!(double_near(result.hue(), 50.0, 10.0));
        assert!(double_near(result.chroma(), 60.0, 0.5));
        assert!(result.tone() > 0.0);
        assert!(result.tone() < 100.0);
    }

    #[test]
    fn test_key_color_unusually_high_chroma() {
        let palette = TonalPalette::from_hue_and_chroma(149.0, 200.0);
        let result = palette.key_color();

        assert!(double_near(result.hue(), 149.0, 10.0));
        assert!(result.chroma() > 89.0);
        assert!(result.tone() > 0.0);
        assert!(result.tone() < 100.0);
    }

    #[test]
    fn test_key_color_unusually_low_chroma() {
        let palette = TonalPalette::from_hue_and_chroma(50.0, 3.0);
        let result = palette.key_color();

        assert!(double_near(result.hue(), 50.0, 10.0));
        assert!(double_near(result.chroma(), 3.0, 0.5));
        assert!(double_near(result.tone(), 50.0, 0.5));
    }
}
