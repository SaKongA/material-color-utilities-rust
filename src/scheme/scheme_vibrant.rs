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

//! Vibrant dynamic color scheme.

use crate::cam::Hct;
use crate::dynamiccolor::{DynamicScheme, Variant};
use crate::palettes::TonalPalette;

const HUES: [f64; 9] = [0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0];
const SECONDARY_ROTATIONS: [f64; 9] = [18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0];
const TERTIARY_ROTATIONS: [f64; 9] = [35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0];

/// A dynamic color scheme with high chroma and punchy accent colors.
pub struct SchemeVibrant;

#[allow(clippy::new_ret_no_self)]
impl SchemeVibrant {
    /// Creates a vibrant scheme with custom contrast level.
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        let hue = source_color_hct.hue();
        DynamicScheme::new(
            source_color_hct,
            Variant::Vibrant,
            contrast_level,
            is_dark,
            TonalPalette::from_hue_and_chroma(hue, 200.0),
            TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(source_color_hct, &HUES, &SECONDARY_ROTATIONS),
                24.0,
            ),
            TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(source_color_hct, &HUES, &TERTIARY_ROTATIONS),
                32.0,
            ),
            TonalPalette::from_hue_and_chroma(hue, 10.0),
            TonalPalette::from_hue_and_chroma(hue, 12.0),
            None,
        )
    }

    /// Creates a vibrant scheme with default contrast level (0.0).
    pub fn with_default_contrast(source_color_hct: Hct, is_dark: bool) -> DynamicScheme {
        Self::new(source_color_hct, is_dark, 0.0)
    }
}
