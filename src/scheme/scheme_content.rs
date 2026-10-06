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

//! Content dynamic color scheme.

use crate::cam::Hct;
use crate::dislike::fix_if_disliked;
use crate::dynamiccolor::{DynamicScheme, Variant};
use crate::palettes::TonalPalette;
use crate::temperature::TemperatureCache;

/// A dynamic color scheme designed for media content where content colors dominate.
pub struct SchemeContent;

#[allow(clippy::new_ret_no_self)]
impl SchemeContent {
    /// Creates a content scheme with custom contrast level.
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        let hue = source_color_hct.hue();
        let chroma = source_color_hct.chroma();
        let mut temp_cache = TemperatureCache::new(source_color_hct);
        let analogous = temp_cache.analogous_colors(3, 6);
        let tertiary_color = analogous[2];

        DynamicScheme::new(
            source_color_hct,
            Variant::Content,
            contrast_level,
            is_dark,
            TonalPalette::from_hue_and_chroma(hue, chroma),
            TonalPalette::from_hue_and_chroma(hue, (chroma - 32.0).max(chroma * 0.5)),
            TonalPalette::from_hct(fix_if_disliked(tertiary_color)),
            TonalPalette::from_hue_and_chroma(hue, chroma / 8.0),
            TonalPalette::from_hue_and_chroma(hue, chroma / 8.0 + 4.0),
            None,
        )
    }

    /// Creates a content scheme with default contrast level (0.0).
    pub fn with_default_contrast(source_color_hct: Hct, is_dark: bool) -> DynamicScheme {
        Self::new(source_color_hct, is_dark, 0.0)
    }
}
