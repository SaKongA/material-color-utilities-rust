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

//! Monochrome dynamic color scheme.

use crate::cam::Hct;
use crate::dynamiccolor::{DynamicScheme, Variant};
use crate::palettes::TonalPalette;

/// A dynamic color scheme that uses only grayscale shades with no chroma.
pub struct SchemeMonochrome;

#[allow(clippy::new_ret_no_self)]
impl SchemeMonochrome {
    /// Creates a monochrome scheme with custom contrast level.
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        let hue = source_color_hct.hue();
        DynamicScheme::new(
            source_color_hct,
            Variant::Monochrome,
            contrast_level,
            is_dark,
            TonalPalette::from_hue_and_chroma(hue, 0.0),
            TonalPalette::from_hue_and_chroma(hue, 0.0),
            TonalPalette::from_hue_and_chroma(hue, 0.0),
            TonalPalette::from_hue_and_chroma(hue, 0.0),
            TonalPalette::from_hue_and_chroma(hue, 0.0),
            None,
        )
    }

    /// Creates a monochrome scheme with default contrast level (0.0).
    pub fn with_default_contrast(source_color_hct: Hct, is_dark: bool) -> DynamicScheme {
        Self::new(source_color_hct, is_dark, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamiccolor::MaterialDynamicColors;

    fn near(a: f64, b: f64, delta: f64) -> bool {
        (a - b).abs() <= delta
    }

    #[test]
    fn test_dark_theme_monochrome_spec() {
        let scheme = SchemeMonochrome::new(Hct::from_int(0xff0000ff), true, 0.0);
        assert!(near(MaterialDynamicColors::primary().get_hct(&scheme).tone(), 100.0, 1.0));
        assert!(near(MaterialDynamicColors::on_primary().get_hct(&scheme).tone(), 10.0, 1.0));
        assert!(near(MaterialDynamicColors::primary_container().get_hct(&scheme).tone(), 85.0, 1.0));
        assert!(near(MaterialDynamicColors::on_primary_container().get_hct(&scheme).tone(), 0.0, 1.0));
        assert!(near(MaterialDynamicColors::secondary().get_hct(&scheme).tone(), 80.0, 1.0));
        assert!(near(MaterialDynamicColors::on_secondary().get_hct(&scheme).tone(), 10.0, 1.0));
        assert!(near(MaterialDynamicColors::secondary_container().get_hct(&scheme).tone(), 30.0, 1.0));
        assert!(near(MaterialDynamicColors::on_secondary_container().get_hct(&scheme).tone(), 90.0, 1.0));
        assert!(near(MaterialDynamicColors::tertiary().get_hct(&scheme).tone(), 90.0, 1.0));
        assert!(near(MaterialDynamicColors::on_tertiary().get_hct(&scheme).tone(), 10.0, 1.0));
        assert!(near(MaterialDynamicColors::tertiary_container().get_hct(&scheme).tone(), 60.0, 1.0));
        assert!(near(MaterialDynamicColors::on_tertiary_container().get_hct(&scheme).tone(), 0.0, 1.0));
    }

    #[test]
    fn test_light_theme_monochrome_spec() {
        let scheme = SchemeMonochrome::new(Hct::from_int(0xff0000ff), false, 0.0);
        assert!(near(MaterialDynamicColors::primary().get_hct(&scheme).tone(), 0.0, 1.0));
        assert!(near(MaterialDynamicColors::on_primary().get_hct(&scheme).tone(), 90.0, 1.0));
        assert!(near(MaterialDynamicColors::primary_container().get_hct(&scheme).tone(), 25.0, 1.0));
        assert!(near(MaterialDynamicColors::on_primary_container().get_hct(&scheme).tone(), 100.0, 1.0));
        assert!(near(MaterialDynamicColors::secondary().get_hct(&scheme).tone(), 40.0, 1.0));
        assert!(near(MaterialDynamicColors::on_secondary().get_hct(&scheme).tone(), 100.0, 1.0));
        assert!(near(MaterialDynamicColors::secondary_container().get_hct(&scheme).tone(), 85.0, 1.0));
        assert!(near(MaterialDynamicColors::on_secondary_container().get_hct(&scheme).tone(), 10.0, 1.0));
        assert!(near(MaterialDynamicColors::tertiary().get_hct(&scheme).tone(), 25.0, 1.0));
        assert!(near(MaterialDynamicColors::on_tertiary().get_hct(&scheme).tone(), 90.0, 1.0));
        assert!(near(MaterialDynamicColors::tertiary_container().get_hct(&scheme).tone(), 49.0, 1.0));
        assert!(near(MaterialDynamicColors::on_tertiary_container().get_hct(&scheme).tone(), 100.0, 1.0));
    }
}
