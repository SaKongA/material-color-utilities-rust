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

//! Expressive dynamic color scheme.

use crate::cam::Hct;
use crate::dynamiccolor::{DynamicScheme, Variant};
use crate::palettes::TonalPalette;
use crate::utils::sanitize_degrees_double;

const HUES: [f64; 9] = [0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0];
const SECONDARY_ROTATIONS: [f64; 9] = [45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0];
const TERTIARY_ROTATIONS: [f64; 9] = [120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0];

/// A playful dynamic color scheme with rotated secondary and tertiary hues.
pub struct SchemeExpressive;

#[allow(clippy::new_ret_no_self)]
impl SchemeExpressive {
    /// Creates an expressive scheme with custom contrast level.
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        let hue = source_color_hct.hue();
        DynamicScheme::new(
            source_color_hct,
            Variant::Expressive,
            contrast_level,
            is_dark,
            TonalPalette::from_hue_and_chroma(sanitize_degrees_double(hue + 240.0), 40.0),
            TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(source_color_hct, &HUES, &SECONDARY_ROTATIONS),
                24.0,
            ),
            TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(source_color_hct, &HUES, &TERTIARY_ROTATIONS),
                32.0,
            ),
            TonalPalette::from_hue_and_chroma(sanitize_degrees_double(hue + 15.0), 8.0),
            TonalPalette::from_hue_and_chroma(sanitize_degrees_double(hue + 15.0), 12.0),
            None,
        )
    }

    /// Creates an expressive scheme with default contrast level (0.0).
    pub fn with_default_contrast(source_color_hct: Hct, is_dark: bool) -> DynamicScheme {
        Self::new(source_color_hct, is_dark, 0.0)
    }
}
