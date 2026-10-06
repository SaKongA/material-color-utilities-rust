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

//! Neutral dynamic color scheme.

use crate::cam::Hct;
use crate::dynamiccolor::{DynamicScheme, Variant};
use crate::palettes::TonalPalette;

/// A dynamic color scheme with near-grayscale saturation.
pub struct SchemeNeutral;

#[allow(clippy::new_ret_no_self)]
impl SchemeNeutral {
    /// Creates a neutral scheme with custom contrast level.
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        let hue = source_color_hct.hue();
        DynamicScheme::new(
            source_color_hct,
            Variant::Neutral,
            contrast_level,
            is_dark,
            TonalPalette::from_hue_and_chroma(hue, 12.0),
            TonalPalette::from_hue_and_chroma(hue, 8.0),
            TonalPalette::from_hue_and_chroma(hue, 16.0),
            TonalPalette::from_hue_and_chroma(hue, 2.0),
            TonalPalette::from_hue_and_chroma(hue, 2.0),
            None,
        )
    }

    /// Creates a neutral scheme with default contrast level (0.0).
    pub fn with_default_contrast(source_color_hct: Hct, is_dark: bool) -> DynamicScheme {
        Self::new(source_color_hct, is_dark, 0.0)
    }
}
