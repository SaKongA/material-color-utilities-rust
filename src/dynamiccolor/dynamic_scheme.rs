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

//! Dynamic color scheme containing the full set of tonal palettes and evaluated colors.

use crate::cam::Hct;
use crate::palettes::TonalPalette;
use crate::utils::{sanitize_degrees_double, Argb};

use super::material_dynamic_colors::MaterialDynamicColors;
use super::variant::Variant;

/// Provides visually balanced color schemes based on a source color and variant style.
#[derive(Debug, Clone)]
pub struct DynamicScheme {
    /// The source color seed for the scheme.
    pub source_color_hct: Hct,
    /// The variant algorithm used.
    pub variant: Variant,
    /// Whether this is a dark theme.
    pub is_dark: bool,
    /// User contrast level: -1.0 (min), 0.0 (normal), 0.5 (medium), 1.0 (max).
    pub contrast_level: f64,

    /// Tonal palette for primary accents.
    pub primary_palette: TonalPalette,
    /// Tonal palette for secondary accents.
    pub secondary_palette: TonalPalette,
    /// Tonal palette for tertiary accents.
    pub tertiary_palette: TonalPalette,
    /// Tonal palette for neutral background and surface colors.
    pub neutral_palette: TonalPalette,
    /// Tonal palette for neutral variant surfaces and outlines.
    pub neutral_variant_palette: TonalPalette,
    /// Tonal palette for error indicators.
    pub error_palette: TonalPalette,
}

impl DynamicScheme {
    /// Creates a new `DynamicScheme`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_color_hct: Hct,
        variant: Variant,
        contrast_level: f64,
        is_dark: bool,
        primary_palette: TonalPalette,
        secondary_palette: TonalPalette,
        tertiary_palette: TonalPalette,
        neutral_palette: TonalPalette,
        neutral_variant_palette: TonalPalette,
        error_palette: Option<TonalPalette>,
    ) -> Self {
        Self {
            source_color_hct,
            variant,
            is_dark,
            contrast_level,
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette: error_palette
                .unwrap_or_else(|| TonalPalette::from_hue_and_chroma(25.0, 84.0)),
        }
    }

    /// Creates a monochrome scheme.
    pub fn monochrome(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeMonochrome::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a tonal spot scheme (default Material 3).
    pub fn tonal_spot(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeTonalSpot::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a vibrant scheme.
    pub fn vibrant(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeVibrant::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates an expressive scheme.
    pub fn expressive(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeExpressive::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a fidelity scheme.
    pub fn fidelity(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeFidelity::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a content scheme.
    pub fn content(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeContent::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a neutral scheme.
    pub fn neutral(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeNeutral::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a rainbow scheme.
    pub fn rainbow(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeRainbow::new(source_color_hct, is_dark, contrast_level)
    }

    /// Creates a fruit salad scheme.
    pub fn fruit_salad(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> Self {
        crate::scheme::SchemeFruitSalad::new(source_color_hct, is_dark, contrast_level)
    }

    /// Rotates a hue according to a mapping of hue ranges to rotation degrees.
    pub fn get_rotated_hue(source_color: Hct, hues: &[f64], rotations: &[f64]) -> f64 {
        let source_hue = source_color.hue();
        if rotations.len() == 1 {
            return sanitize_degrees_double(source_hue + rotations[0]);
        }
        let size = hues.len();
        if size >= 2 {
            for i in 0..=(size - 2) {
                let this_hue = hues[i];
                let next_hue = hues[i + 1];
                if this_hue < source_hue && source_hue < next_hue {
                    return sanitize_degrees_double(source_hue + rotations[i]);
                }
            }
        }
        source_hue
    }

    /// Returns the ARGB integer of the source seed color.
    pub fn source_color_argb(&self) -> Argb {
        self.source_color_hct.to_int()
    }

    // -------------------------------------------------------------------------
    // Evaluated Role Colors (ARGB)
    // -------------------------------------------------------------------------

    pub fn primary_palette_key_color(&self) -> Argb {
        MaterialDynamicColors::primary_palette_key_color().get_argb(self)
    }

    pub fn secondary_palette_key_color(&self) -> Argb {
        MaterialDynamicColors::secondary_palette_key_color().get_argb(self)
    }

    pub fn tertiary_palette_key_color(&self) -> Argb {
        MaterialDynamicColors::tertiary_palette_key_color().get_argb(self)
    }

    pub fn neutral_palette_key_color(&self) -> Argb {
        MaterialDynamicColors::neutral_palette_key_color().get_argb(self)
    }

    pub fn neutral_variant_palette_key_color(&self) -> Argb {
        MaterialDynamicColors::neutral_variant_palette_key_color().get_argb(self)
    }

    pub fn background(&self) -> Argb {
        MaterialDynamicColors::background().get_argb(self)
    }

    pub fn on_background(&self) -> Argb {
        MaterialDynamicColors::on_background().get_argb(self)
    }

    pub fn surface(&self) -> Argb {
        MaterialDynamicColors::surface().get_argb(self)
    }

    pub fn surface_dim(&self) -> Argb {
        MaterialDynamicColors::surface_dim().get_argb(self)
    }

    pub fn surface_bright(&self) -> Argb {
        MaterialDynamicColors::surface_bright().get_argb(self)
    }

    pub fn surface_container_lowest(&self) -> Argb {
        MaterialDynamicColors::surface_container_lowest().get_argb(self)
    }

    pub fn surface_container_low(&self) -> Argb {
        MaterialDynamicColors::surface_container_low().get_argb(self)
    }

    pub fn surface_container(&self) -> Argb {
        MaterialDynamicColors::surface_container().get_argb(self)
    }

    pub fn surface_container_high(&self) -> Argb {
        MaterialDynamicColors::surface_container_high().get_argb(self)
    }

    pub fn surface_container_highest(&self) -> Argb {
        MaterialDynamicColors::surface_container_highest().get_argb(self)
    }

    pub fn on_surface(&self) -> Argb {
        MaterialDynamicColors::on_surface().get_argb(self)
    }

    pub fn surface_variant(&self) -> Argb {
        MaterialDynamicColors::surface_variant().get_argb(self)
    }

    pub fn on_surface_variant(&self) -> Argb {
        MaterialDynamicColors::on_surface_variant().get_argb(self)
    }

    pub fn inverse_surface(&self) -> Argb {
        MaterialDynamicColors::inverse_surface().get_argb(self)
    }

    pub fn inverse_on_surface(&self) -> Argb {
        MaterialDynamicColors::inverse_on_surface().get_argb(self)
    }

    pub fn outline(&self) -> Argb {
        MaterialDynamicColors::outline().get_argb(self)
    }

    pub fn outline_variant(&self) -> Argb {
        MaterialDynamicColors::outline_variant().get_argb(self)
    }

    pub fn shadow(&self) -> Argb {
        MaterialDynamicColors::shadow().get_argb(self)
    }

    pub fn scrim(&self) -> Argb {
        MaterialDynamicColors::scrim().get_argb(self)
    }

    pub fn surface_tint(&self) -> Argb {
        MaterialDynamicColors::surface_tint().get_argb(self)
    }

    pub fn primary(&self) -> Argb {
        MaterialDynamicColors::primary().get_argb(self)
    }

    pub fn on_primary(&self) -> Argb {
        MaterialDynamicColors::on_primary().get_argb(self)
    }

    pub fn primary_container(&self) -> Argb {
        MaterialDynamicColors::primary_container().get_argb(self)
    }

    pub fn on_primary_container(&self) -> Argb {
        MaterialDynamicColors::on_primary_container().get_argb(self)
    }

    pub fn inverse_primary(&self) -> Argb {
        MaterialDynamicColors::inverse_primary().get_argb(self)
    }

    pub fn secondary(&self) -> Argb {
        MaterialDynamicColors::secondary().get_argb(self)
    }

    pub fn on_secondary(&self) -> Argb {
        MaterialDynamicColors::on_secondary().get_argb(self)
    }

    pub fn secondary_container(&self) -> Argb {
        MaterialDynamicColors::secondary_container().get_argb(self)
    }

    pub fn on_secondary_container(&self) -> Argb {
        MaterialDynamicColors::on_secondary_container().get_argb(self)
    }

    pub fn tertiary(&self) -> Argb {
        MaterialDynamicColors::tertiary().get_argb(self)
    }

    pub fn on_tertiary(&self) -> Argb {
        MaterialDynamicColors::on_tertiary().get_argb(self)
    }

    pub fn tertiary_container(&self) -> Argb {
        MaterialDynamicColors::tertiary_container().get_argb(self)
    }

    pub fn on_tertiary_container(&self) -> Argb {
        MaterialDynamicColors::on_tertiary_container().get_argb(self)
    }

    pub fn error(&self) -> Argb {
        MaterialDynamicColors::error().get_argb(self)
    }

    pub fn on_error(&self) -> Argb {
        MaterialDynamicColors::on_error().get_argb(self)
    }

    pub fn error_container(&self) -> Argb {
        MaterialDynamicColors::error_container().get_argb(self)
    }

    pub fn on_error_container(&self) -> Argb {
        MaterialDynamicColors::on_error_container().get_argb(self)
    }

    pub fn primary_fixed(&self) -> Argb {
        MaterialDynamicColors::primary_fixed().get_argb(self)
    }

    pub fn primary_fixed_dim(&self) -> Argb {
        MaterialDynamicColors::primary_fixed_dim().get_argb(self)
    }

    pub fn on_primary_fixed(&self) -> Argb {
        MaterialDynamicColors::on_primary_fixed().get_argb(self)
    }

    pub fn on_primary_fixed_variant(&self) -> Argb {
        MaterialDynamicColors::on_primary_fixed_variant().get_argb(self)
    }

    pub fn secondary_fixed(&self) -> Argb {
        MaterialDynamicColors::secondary_fixed().get_argb(self)
    }

    pub fn secondary_fixed_dim(&self) -> Argb {
        MaterialDynamicColors::secondary_fixed_dim().get_argb(self)
    }

    pub fn on_secondary_fixed(&self) -> Argb {
        MaterialDynamicColors::on_secondary_fixed().get_argb(self)
    }

    pub fn on_secondary_fixed_variant(&self) -> Argb {
        MaterialDynamicColors::on_secondary_fixed_variant().get_argb(self)
    }

    pub fn tertiary_fixed(&self) -> Argb {
        MaterialDynamicColors::tertiary_fixed().get_argb(self)
    }

    pub fn tertiary_fixed_dim(&self) -> Argb {
        MaterialDynamicColors::tertiary_fixed_dim().get_argb(self)
    }

    pub fn on_tertiary_fixed(&self) -> Argb {
        MaterialDynamicColors::on_tertiary_fixed().get_argb(self)
    }

    pub fn on_tertiary_fixed_variant(&self) -> Argb {
        MaterialDynamicColors::on_tertiary_fixed_variant().get_argb(self)
    }

    // -------------------------------------------------------------------------
    // C++ compatibility getters (`get_*`)
    // -------------------------------------------------------------------------

    pub fn get_primary_palette_key_color(&self) -> Argb {
        self.primary_palette_key_color()
    }
    pub fn get_secondary_palette_key_color(&self) -> Argb {
        self.secondary_palette_key_color()
    }
    pub fn get_tertiary_palette_key_color(&self) -> Argb {
        self.tertiary_palette_key_color()
    }
    pub fn get_neutral_palette_key_color(&self) -> Argb {
        self.neutral_palette_key_color()
    }
    pub fn get_neutral_variant_palette_key_color(&self) -> Argb {
        self.neutral_variant_palette_key_color()
    }
    pub fn get_background(&self) -> Argb {
        self.background()
    }
    pub fn get_on_background(&self) -> Argb {
        self.on_background()
    }
    pub fn get_surface(&self) -> Argb {
        self.surface()
    }
    pub fn get_surface_dim(&self) -> Argb {
        self.surface_dim()
    }
    pub fn get_surface_bright(&self) -> Argb {
        self.surface_bright()
    }
    pub fn get_surface_container_lowest(&self) -> Argb {
        self.surface_container_lowest()
    }
    pub fn get_surface_container_low(&self) -> Argb {
        self.surface_container_low()
    }
    pub fn get_surface_container(&self) -> Argb {
        self.surface_container()
    }
    pub fn get_surface_container_high(&self) -> Argb {
        self.surface_container_high()
    }
    pub fn get_surface_container_highest(&self) -> Argb {
        self.surface_container_highest()
    }
    pub fn get_on_surface(&self) -> Argb {
        self.on_surface()
    }
    pub fn get_surface_variant(&self) -> Argb {
        self.surface_variant()
    }
    pub fn get_on_surface_variant(&self) -> Argb {
        self.on_surface_variant()
    }
    pub fn get_inverse_surface(&self) -> Argb {
        self.inverse_surface()
    }
    pub fn get_inverse_on_surface(&self) -> Argb {
        self.inverse_on_surface()
    }
    pub fn get_outline(&self) -> Argb {
        self.outline()
    }
    pub fn get_outline_variant(&self) -> Argb {
        self.outline_variant()
    }
    pub fn get_shadow(&self) -> Argb {
        self.shadow()
    }
    pub fn get_scrim(&self) -> Argb {
        self.scrim()
    }
    pub fn get_surface_tint(&self) -> Argb {
        self.surface_tint()
    }
    pub fn get_primary(&self) -> Argb {
        self.primary()
    }
    pub fn get_on_primary(&self) -> Argb {
        self.on_primary()
    }
    pub fn get_primary_container(&self) -> Argb {
        self.primary_container()
    }
    pub fn get_on_primary_container(&self) -> Argb {
        self.on_primary_container()
    }
    pub fn get_inverse_primary(&self) -> Argb {
        self.inverse_primary()
    }
    pub fn get_secondary(&self) -> Argb {
        self.secondary()
    }
    pub fn get_on_secondary(&self) -> Argb {
        self.on_secondary()
    }
    pub fn get_secondary_container(&self) -> Argb {
        self.secondary_container()
    }
    pub fn get_on_secondary_container(&self) -> Argb {
        self.on_secondary_container()
    }
    pub fn get_tertiary(&self) -> Argb {
        self.tertiary()
    }
    pub fn get_on_tertiary(&self) -> Argb {
        self.on_tertiary()
    }
    pub fn get_tertiary_container(&self) -> Argb {
        self.tertiary_container()
    }
    pub fn get_on_tertiary_container(&self) -> Argb {
        self.on_tertiary_container()
    }
    pub fn get_error(&self) -> Argb {
        self.error()
    }
    pub fn get_on_error(&self) -> Argb {
        self.on_error()
    }
    pub fn get_error_container(&self) -> Argb {
        self.error_container()
    }
    pub fn get_on_error_container(&self) -> Argb {
        self.on_error_container()
    }
    pub fn get_primary_fixed(&self) -> Argb {
        self.primary_fixed()
    }
    pub fn get_primary_fixed_dim(&self) -> Argb {
        self.primary_fixed_dim()
    }
    pub fn get_on_primary_fixed(&self) -> Argb {
        self.on_primary_fixed()
    }
    pub fn get_on_primary_fixed_variant(&self) -> Argb {
        self.on_primary_fixed_variant()
    }
    pub fn get_secondary_fixed(&self) -> Argb {
        self.secondary_fixed()
    }
    pub fn get_secondary_fixed_dim(&self) -> Argb {
        self.secondary_fixed_dim()
    }
    pub fn get_on_secondary_fixed(&self) -> Argb {
        self.on_secondary_fixed()
    }
    pub fn get_on_secondary_fixed_variant(&self) -> Argb {
        self.on_secondary_fixed_variant()
    }
    pub fn get_tertiary_fixed(&self) -> Argb {
        self.tertiary_fixed()
    }
    pub fn get_tertiary_fixed_dim(&self) -> Argb {
        self.tertiary_fixed_dim()
    }
    pub fn get_on_tertiary_fixed(&self) -> Argb {
        self.on_tertiary_fixed()
    }
    pub fn get_on_tertiary_fixed_variant(&self) -> Argb {
        self.on_tertiary_fixed_variant()
    }
}
