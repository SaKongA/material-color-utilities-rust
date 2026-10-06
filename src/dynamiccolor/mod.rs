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

//! Dynamic color architecture for Material Design 3.
//!
//! Provides algorithmic generation of UI colors that adapt dynamically based on
//! user contrast preferences, light/dark mode, and surface elevation relationships.

pub mod contrast_curve;
pub mod dynamic_color;
pub mod dynamic_scheme;
pub mod material_dynamic_colors;
pub mod tone_delta_pair;
pub mod variant;

pub use contrast_curve::ContrastCurve;
pub use dynamic_color::{
    enable_light_foreground, foreground_tone, tone_allows_light_foreground,
    tone_prefers_light_foreground, DynamicColor, DynamicColorFn, PaletteFn, ToneDeltaPairFn, ToneFn,
};
pub use dynamic_scheme::DynamicScheme;
pub use material_dynamic_colors::MaterialDynamicColors;
pub use tone_delta_pair::{ToneDeltaPair, TonePolarity};
pub use variant::Variant;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cam::Hct;
    use crate::contrast::ratio_of_tones;
    use crate::scheme::SchemeTonalSpot;

    #[test]
    fn test_tone_preferences() {
        assert!(tone_prefers_light_foreground(59.0));
        assert!(!tone_prefers_light_foreground(60.0));
        assert!(tone_allows_light_foreground(49.0));
        assert!(!tone_allows_light_foreground(50.0));
    }

    #[test]
    fn test_foreground_tone_meets_ratio() {
        let bg_tone = 30.0;
        let fg = foreground_tone(bg_tone, 4.5);
        assert!(ratio_of_tones(fg, bg_tone) >= 4.5);
    }

    #[test]
    fn test_enable_light_foreground() {
        assert_eq!(enable_light_foreground(55.0), 49.0);
        assert_eq!(enable_light_foreground(40.0), 40.0);
        assert_eq!(enable_light_foreground(70.0), 70.0);
    }

    #[test]
    fn test_fixed_colors_resolution() {
        let scheme = SchemeTonalSpot::new(Hct::from_int(0xff6750a4), false, 0.0);
        let pf = MaterialDynamicColors::primary_fixed().get_argb(&scheme);
        let opf = MaterialDynamicColors::on_primary_fixed().get_argb(&scheme);
        let pf_tone = Hct::from_int(pf).tone();
        let opf_tone = Hct::from_int(opf).tone();
        assert!(ratio_of_tones(pf_tone, opf_tone) >= 4.5);
    }
}
