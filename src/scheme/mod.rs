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

//! Standard Material 3 dynamic color scheme presets.

pub mod scheme_content;
pub mod scheme_expressive;
pub mod scheme_fidelity;
pub mod scheme_fruit_salad;
pub mod scheme_monochrome;
pub mod scheme_neutral;
pub mod scheme_rainbow;
pub mod scheme_tonal_spot;
pub mod scheme_vibrant;

pub use scheme_content::SchemeContent;
pub use scheme_expressive::SchemeExpressive;
pub use scheme_fidelity::SchemeFidelity;
pub use scheme_fruit_salad::SchemeFruitSalad;
pub use scheme_monochrome::SchemeMonochrome;
pub use scheme_neutral::SchemeNeutral;
pub use scheme_rainbow::SchemeRainbow;
pub use scheme_tonal_spot::SchemeTonalSpot;
pub use scheme_vibrant::SchemeVibrant;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cam::Hct;

    #[test]
    fn test_all_scheme_presets_generate_valid_colors() {
        let seed = Hct::from_int(0xff4285f4); // Google Blue

        let schemes = [
            SchemeTonalSpot::new(seed, false, 0.0),
            SchemeTonalSpot::new(seed, true, 0.0),
            SchemeVibrant::new(seed, false, 0.0),
            SchemeVibrant::new(seed, true, 0.0),
            SchemeExpressive::new(seed, false, 0.0),
            SchemeExpressive::new(seed, true, 0.0),
            SchemeFidelity::new(seed, false, 0.0),
            SchemeFidelity::new(seed, true, 0.0),
            SchemeContent::new(seed, false, 0.0),
            SchemeContent::new(seed, true, 0.0),
            SchemeNeutral::new(seed, false, 0.0),
            SchemeNeutral::new(seed, true, 0.0),
            SchemeRainbow::new(seed, false, 0.0),
            SchemeRainbow::new(seed, true, 0.0),
            SchemeFruitSalad::new(seed, false, 0.0),
            SchemeFruitSalad::new(seed, true, 0.0),
            SchemeMonochrome::new(seed, false, 0.0),
            SchemeMonochrome::new(seed, true, 0.0),
        ];

        for scheme in &schemes {
            // Verify core palette key colors are valid non-zero ARGB
            assert_ne!(scheme.primary_palette_key_color(), 0);
            assert_ne!(scheme.secondary_palette_key_color(), 0);
            assert_ne!(scheme.tertiary_palette_key_color(), 0);
            assert_ne!(scheme.neutral_palette_key_color(), 0);
            assert_ne!(scheme.neutral_variant_palette_key_color(), 0);

            // Verify core UI role colors are generated
            assert_ne!(scheme.primary(), 0);
            assert_ne!(scheme.on_primary(), 0);
            assert_ne!(scheme.primary_container(), 0);
            assert_ne!(scheme.on_primary_container(), 0);
            assert_ne!(scheme.surface(), 0);
            assert_ne!(scheme.on_surface(), 0);
            assert_ne!(scheme.background(), 0);
            assert_ne!(scheme.on_background(), 0);
            assert_ne!(scheme.outline(), 0);
            assert_ne!(scheme.error(), 0);
            assert_ne!(scheme.on_error(), 0);
        }
    }

    #[test]
    fn test_contrast_level_variations() {
        let seed = Hct::from_int(0xffeb0052);
        for contrast in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let scheme_light = SchemeTonalSpot::new(seed, false, contrast);
            let scheme_dark = SchemeTonalSpot::new(seed, true, contrast);

            assert_ne!(scheme_light.primary(), 0);
            assert_ne!(scheme_dark.primary(), 0);
        }
    }
}
