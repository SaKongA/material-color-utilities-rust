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

//! Standard Material 3 dynamic color tokens and definitions.

use std::f64::consts::PI;
use std::sync::Arc;

use crate::cam::cam::{cam_from_int, cam_from_xyz_and_viewing_conditions, Cam};
use crate::cam::viewing_conditions::DEFAULT_VIEWING_CONDITIONS;
use crate::cam::{Hct, ViewingConditions};
use crate::dislike::fix_if_disliked;
use crate::utils::{lstar_from_y, signum, Vec3};

use super::contrast_curve::ContrastCurve;
use super::dynamic_color::{foreground_tone, DynamicColor};
use super::dynamic_scheme::DynamicScheme;
use super::tone_delta_pair::{ToneDeltaPair, TonePolarity};
use super::variant::Variant;

/// Checks if the scheme is fidelity or content variant.
#[inline]
pub fn is_fidelity(scheme: &DynamicScheme) -> bool {
    scheme.variant == Variant::Fidelity || scheme.variant == Variant::Content
}

/// Checks if the scheme is monochrome variant.
#[inline]
pub fn is_monochrome(scheme: &DynamicScheme) -> bool {
    scheme.variant == Variant::Monochrome
}

/// Converts a CAM16 color to XYZ coordinates in the specified viewing conditions.
pub fn xyz_in_viewing_conditions(cam: Cam, viewing_conditions: ViewingConditions) -> Vec3 {
    let alpha = if cam.chroma == 0.0 || cam.j == 0.0 {
        0.0
    } else {
        cam.chroma / (cam.j / 100.0).sqrt()
    };

    let t = (alpha
        / (1.64 - 0.29f64.powf(viewing_conditions.background_y_to_white_point_y)).powf(0.73))
    .powf(1.0 / 0.9);
    let h_rad = cam.hue * PI / 180.0;

    let e_hue = 0.25 * ((h_rad + 2.0).cos() + 3.8);
    let ac = viewing_conditions.aw
        * (cam.j / 100.0).powf(1.0 / viewing_conditions.c / viewing_conditions.z);
    let p1 = e_hue * (50000.0 / 13.0) * viewing_conditions.n_c * viewing_conditions.ncb;

    let p2 = ac / viewing_conditions.nbb;

    let h_sin = h_rad.sin();
    let h_cos = h_rad.cos();

    let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * h_cos + 108.0 * t * h_sin);
    let a = gamma * h_cos;
    let b = gamma * h_sin;
    let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
    let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
    let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

    let r_c_base = (27.13 * r_a.abs()) / (400.0 - r_a.abs());
    let r_c_base = r_c_base.max(0.0);
    let r_c = signum(r_a) as f64 * (100.0 / viewing_conditions.fl) * r_c_base.powf(1.0 / 0.42);

    let g_c_base = (27.13 * g_a.abs()) / (400.0 - g_a.abs());
    let g_c_base = g_c_base.max(0.0);
    let g_c = signum(g_a) as f64 * (100.0 / viewing_conditions.fl) * g_c_base.powf(1.0 / 0.42);

    let b_c_base = (27.13 * b_a.abs()) / (400.0 - b_a.abs());
    let b_c_base = b_c_base.max(0.0);
    let b_c = signum(b_a) as f64 * (100.0 / viewing_conditions.fl) * b_c_base.powf(1.0 / 0.42);

    let r_f = r_c / viewing_conditions.rgb_d[0];
    let g_f = g_c / viewing_conditions.rgb_d[1];
    let b_f = b_c / viewing_conditions.rgb_d[2];

    let x = 1.86206786 * r_f - 1.01125463 * g_f + 0.14918677 * b_f;
    let y = 0.38752654 * r_f + 0.62144744 * g_f - 0.00897398 * b_f;
    let z = -0.01584150 * r_f - 0.03412294 * g_f + 1.04996444 * b_f;

    Vec3::new(x, y, z)
}

/// Recasts an HCT color in specified viewing conditions to default viewing conditions.
pub fn in_viewing_conditions(hct: Hct, vc: ViewingConditions) -> Hct {
    let cam16 = cam_from_int(hct.to_int());
    let viewed_in_vc = xyz_in_viewing_conditions(cam16, vc);
    let recast_in_vc = cam_from_xyz_and_viewing_conditions(
        viewed_in_vc.a,
        viewed_in_vc.b,
        viewed_in_vc.c,
        &DEFAULT_VIEWING_CONDITIONS,
    );
    Hct::new(
        recast_in_vc.hue,
        recast_in_vc.chroma,
        lstar_from_y(viewed_in_vc.b),
    )
}

/// Finds the tone that achieves the desired chroma for a hue, moving in steps of 1 tone.
pub fn find_desired_chroma_by_tone(
    hue: f64,
    chroma: f64,
    tone: f64,
    by_decreasing_tone: bool,
) -> f64 {
    let mut answer = tone;
    let mut closest_to_chroma = Hct::new(hue, chroma, tone);
    if closest_to_chroma.chroma() < chroma {
        let mut chroma_peak = closest_to_chroma.chroma();
        while closest_to_chroma.chroma() < chroma {
            answer += if by_decreasing_tone { -1.0 } else { 1.0 };
            let potential_solution = Hct::new(hue, chroma, answer);
            if chroma_peak > potential_solution.chroma() {
                break;
            }
            if (potential_solution.chroma() - chroma).abs() < 0.4 {
                break;
            }
            let potential_delta = (potential_solution.chroma() - chroma).abs();
            let current_delta = (closest_to_chroma.chroma() - chroma).abs();
            if potential_delta < current_delta {
                closest_to_chroma = potential_solution;
            }
            chroma_peak = chroma_peak.max(potential_solution.chroma());
        }
    }
    answer
}

/// Helper function to return the highest surface dynamic color for a scheme.
#[inline]
pub fn highest_surface(s: &DynamicScheme) -> DynamicColor {
    if s.is_dark {
        MaterialDynamicColors::surface_bright()
    } else {
        MaterialDynamicColors::surface_dim()
    }
}

/// Named tokens for all standard dynamic color roles in Material Design 3.
pub struct MaterialDynamicColors;

impl MaterialDynamicColors {
    // -------------------------------------------------------------------------
    // Compatibility Palette Keys
    // -------------------------------------------------------------------------

    pub fn primary_palette_key_color() -> DynamicColor {
        DynamicColor::from_palette(
            "primary_palette_key_color",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| s.primary_palette.key_color().tone()),
        )
    }

    pub fn secondary_palette_key_color() -> DynamicColor {
        DynamicColor::from_palette(
            "secondary_palette_key_color",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| s.secondary_palette.key_color().tone()),
        )
    }

    pub fn tertiary_palette_key_color() -> DynamicColor {
        DynamicColor::from_palette(
            "tertiary_palette_key_color",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| s.tertiary_palette.key_color().tone()),
        )
    }

    pub fn neutral_palette_key_color() -> DynamicColor {
        DynamicColor::from_palette(
            "neutral_palette_key_color",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| s.neutral_palette.key_color().tone()),
        )
    }

    pub fn neutral_variant_palette_key_color() -> DynamicColor {
        DynamicColor::from_palette(
            "neutral_variant_palette_key_color",
            Arc::new(|s| s.neutral_variant_palette),
            Arc::new(|s| s.neutral_variant_palette.key_color().tone()),
        )
    }

    // -------------------------------------------------------------------------
    // Surfaces & Background
    // -------------------------------------------------------------------------

    pub fn background() -> DynamicColor {
        DynamicColor::new(
            "background",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn on_background() -> DynamicColor {
        DynamicColor::new(
            "on_background",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 }),
            false,
            Some(Arc::new(|_s| Self::background())),
            None,
            Some(ContrastCurve::new(3.0, 3.0, 4.5, 7.0)),
            None,
        )
    }

    pub fn surface() -> DynamicColor {
        DynamicColor::new(
            "surface",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_dim() -> DynamicColor {
        DynamicColor::new(
            "surface_dim",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    6.0
                } else {
                    ContrastCurve::new(87.0, 87.0, 80.0, 75.0).get(s.contrast_level)
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_bright() -> DynamicColor {
        DynamicColor::new(
            "surface_bright",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(24.0, 24.0, 29.0, 34.0).get(s.contrast_level)
                } else {
                    98.0
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_container_lowest() -> DynamicColor {
        DynamicColor::new(
            "surface_container_lowest",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(4.0, 4.0, 2.0, 0.0).get(s.contrast_level)
                } else {
                    100.0
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_container_low() -> DynamicColor {
        DynamicColor::new(
            "surface_container_low",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(10.0, 10.0, 11.0, 12.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(96.0, 96.0, 96.0, 95.0).get(s.contrast_level)
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_container() -> DynamicColor {
        DynamicColor::new(
            "surface_container",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(12.0, 12.0, 16.0, 20.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(94.0, 94.0, 92.0, 90.0).get(s.contrast_level)
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_container_high() -> DynamicColor {
        DynamicColor::new(
            "surface_container_high",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(17.0, 17.0, 21.0, 25.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(92.0, 92.0, 88.0, 85.0).get(s.contrast_level)
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_container_highest() -> DynamicColor {
        DynamicColor::new(
            "surface_container_highest",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(22.0, 22.0, 26.0, 30.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(90.0, 90.0, 84.0, 80.0).get(s.contrast_level)
                }
            }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn on_surface() -> DynamicColor {
        DynamicColor::new(
            "on_surface",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 }),
            false,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn surface_variant() -> DynamicColor {
        DynamicColor::new(
            "surface_variant",
            Arc::new(|s| s.neutral_variant_palette),
            Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    pub fn on_surface_variant() -> DynamicColor {
        DynamicColor::new(
            "on_surface_variant",
            Arc::new(|s| s.neutral_variant_palette),
            Arc::new(|s| if s.is_dark { 80.0 } else { 30.0 }),
            false,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    pub fn inverse_surface() -> DynamicColor {
        DynamicColor::new(
            "inverse_surface",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 90.0 } else { 20.0 }),
            false,
            None,
            None,
            None,
            None,
        )
    }

    pub fn inverse_on_surface() -> DynamicColor {
        DynamicColor::new(
            "inverse_on_surface",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|s| if s.is_dark { 20.0 } else { 95.0 }),
            false,
            Some(Arc::new(|_s| Self::inverse_surface())),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn outline() -> DynamicColor {
        DynamicColor::new(
            "outline",
            Arc::new(|s| s.neutral_variant_palette),
            Arc::new(|s| if s.is_dark { 60.0 } else { 50.0 }),
            false,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.5, 3.0, 4.5, 7.0)),
            None,
        )
    }

    pub fn outline_variant() -> DynamicColor {
        DynamicColor::new(
            "outline_variant",
            Arc::new(|s| s.neutral_variant_palette),
            Arc::new(|s| if s.is_dark { 30.0 } else { 80.0 }),
            false,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            None,
        )
    }

    pub fn shadow() -> DynamicColor {
        DynamicColor::new(
            "shadow",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|_s| 0.0),
            false,
            None,
            None,
            None,
            None,
        )
    }

    pub fn scrim() -> DynamicColor {
        DynamicColor::new(
            "scrim",
            Arc::new(|s| s.neutral_palette),
            Arc::new(|_s| 0.0),
            false,
            None,
            None,
            None,
            None,
        )
    }

    pub fn surface_tint() -> DynamicColor {
        DynamicColor::new(
            "surface_tint",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 }),
            true,
            None,
            None,
            None,
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Primary Roles
    // -------------------------------------------------------------------------

    pub fn primary() -> DynamicColor {
        DynamicColor::new(
            "primary",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 100.0 } else { 0.0 }
                } else if s.is_dark {
                    80.0
                } else {
                    40.0
                }
            }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::primary_container(),
                    Self::primary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_primary() -> DynamicColor {
        DynamicColor::new(
            "on_primary",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 90.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            }),
            false,
            Some(Arc::new(|_s| Self::primary())),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn primary_container() -> DynamicColor {
        DynamicColor::new(
            "primary_container",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| {
                if is_fidelity(s) {
                    s.source_color_hct.tone()
                } else if is_monochrome(s) {
                    if s.is_dark { 85.0 } else { 25.0 }
                } else if s.is_dark {
                    30.0
                } else {
                    90.0
                }
            }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::primary_container(),
                    Self::primary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_primary_container() -> DynamicColor {
        DynamicColor::new(
            "on_primary_container",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| {
                if is_fidelity(s) {
                    foreground_tone((Self::primary_container().tone)(s), 4.5)
                } else if is_monochrome(s) {
                    if s.is_dark { 0.0 } else { 100.0 }
                } else if s.is_dark {
                    90.0
                } else {
                    30.0
                }
            }),
            false,
            Some(Arc::new(|_s| Self::primary_container())),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    pub fn inverse_primary() -> DynamicColor {
        DynamicColor::new(
            "inverse_primary",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if s.is_dark { 40.0 } else { 80.0 }),
            false,
            Some(Arc::new(|_s| Self::inverse_surface())),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Secondary Roles
    // -------------------------------------------------------------------------

    pub fn secondary() -> DynamicColor {
        DynamicColor::new(
            "secondary",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::secondary_container(),
                    Self::secondary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_secondary() -> DynamicColor {
        DynamicColor::new(
            "on_secondary",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 100.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            }),
            false,
            Some(Arc::new(|_s| Self::secondary())),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn secondary_container() -> DynamicColor {
        DynamicColor::new(
            "secondary_container",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| {
                let initial_tone = if s.is_dark { 30.0 } else { 90.0 };
                if is_monochrome(s) {
                    if s.is_dark { 30.0 } else { 85.0 }
                } else if !is_fidelity(s) {
                    initial_tone
                } else {
                    find_desired_chroma_by_tone(
                        s.secondary_palette.hue(),
                        s.secondary_palette.chroma(),
                        initial_tone,
                        !s.is_dark,
                    )
                }
            }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::secondary_container(),
                    Self::secondary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_secondary_container() -> DynamicColor {
        DynamicColor::new(
            "on_secondary_container",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 10.0 }
                } else if !is_fidelity(s) {
                    if s.is_dark { 90.0 } else { 30.0 }
                } else {
                    foreground_tone((Self::secondary_container().tone)(s), 4.5)
                }
            }),
            false,
            Some(Arc::new(|_s| Self::secondary_container())),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Tertiary Roles
    // -------------------------------------------------------------------------

    pub fn tertiary() -> DynamicColor {
        DynamicColor::new(
            "tertiary",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 25.0 }
                } else if s.is_dark {
                    80.0
                } else {
                    40.0
                }
            }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::tertiary_container(),
                    Self::tertiary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_tertiary() -> DynamicColor {
        DynamicColor::new(
            "on_tertiary",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 90.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            }),
            false,
            Some(Arc::new(|_s| Self::tertiary())),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn tertiary_container() -> DynamicColor {
        DynamicColor::new(
            "tertiary_container",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 60.0 } else { 49.0 }
                } else if !is_fidelity(s) {
                    if s.is_dark { 30.0 } else { 90.0 }
                } else {
                    let proposed_hct =
                        Hct::from_int(s.tertiary_palette.get(s.source_color_hct.tone()));
                    fix_if_disliked(proposed_hct).tone()
                }
            }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::tertiary_container(),
                    Self::tertiary(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_tertiary_container() -> DynamicColor {
        DynamicColor::new(
            "on_tertiary_container",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 0.0 } else { 100.0 }
                } else if !is_fidelity(s) {
                    if s.is_dark { 90.0 } else { 30.0 }
                } else {
                    foreground_tone((Self::tertiary_container().tone)(s), 4.5)
                }
            }),
            false,
            Some(Arc::new(|_s| Self::tertiary_container())),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Error Roles
    // -------------------------------------------------------------------------

    pub fn error() -> DynamicColor {
        DynamicColor::new(
            "error",
            Arc::new(|s| s.error_palette),
            Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::error_container(),
                    Self::error(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_error() -> DynamicColor {
        DynamicColor::new(
            "on_error",
            Arc::new(|s| s.error_palette),
            Arc::new(|s| if s.is_dark { 20.0 } else { 100.0 }),
            false,
            Some(Arc::new(|_s| Self::error())),
            None,
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn error_container() -> DynamicColor {
        DynamicColor::new(
            "error_container",
            Arc::new(|s| s.error_palette),
            Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::error_container(),
                    Self::error(),
                    10.0,
                    TonePolarity::Nearer,
                    false,
                )
            })),
        )
    }

    pub fn on_error_container() -> DynamicColor {
        DynamicColor::new(
            "on_error_container",
            Arc::new(|s| s.error_palette),
            Arc::new(|s| {
                if is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 10.0 }
                } else if s.is_dark {
                    90.0
                } else {
                    30.0
                }
            }),
            false,
            Some(Arc::new(|_s| Self::error_container())),
            None,
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Primary Fixed Roles
    // -------------------------------------------------------------------------

    pub fn primary_fixed() -> DynamicColor {
        DynamicColor::new(
            "primary_fixed",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::primary_fixed(),
                    Self::primary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn primary_fixed_dim() -> DynamicColor {
        DynamicColor::new(
            "primary_fixed_dim",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::primary_fixed(),
                    Self::primary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn on_primary_fixed() -> DynamicColor {
        DynamicColor::new(
            "on_primary_fixed",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            false,
            Some(Arc::new(|_s| Self::primary_fixed_dim())),
            Some(Arc::new(|_s| Self::primary_fixed())),
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn on_primary_fixed_variant() -> DynamicColor {
        DynamicColor::new(
            "on_primary_fixed_variant",
            Arc::new(|s| s.primary_palette),
            Arc::new(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            false,
            Some(Arc::new(|_s| Self::primary_fixed_dim())),
            Some(Arc::new(|_s| Self::primary_fixed())),
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Secondary Fixed Roles
    // -------------------------------------------------------------------------

    pub fn secondary_fixed() -> DynamicColor {
        DynamicColor::new(
            "secondary_fixed",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| if is_monochrome(s) { 80.0 } else { 90.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::secondary_fixed(),
                    Self::secondary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn secondary_fixed_dim() -> DynamicColor {
        DynamicColor::new(
            "secondary_fixed_dim",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| if is_monochrome(s) { 70.0 } else { 80.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::secondary_fixed(),
                    Self::secondary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn on_secondary_fixed() -> DynamicColor {
        DynamicColor::new(
            "on_secondary_fixed",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|_s| 10.0),
            false,
            Some(Arc::new(|_s| Self::secondary_fixed_dim())),
            Some(Arc::new(|_s| Self::secondary_fixed())),
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn on_secondary_fixed_variant() -> DynamicColor {
        DynamicColor::new(
            "on_secondary_fixed_variant",
            Arc::new(|s| s.secondary_palette),
            Arc::new(|s| if is_monochrome(s) { 25.0 } else { 30.0 }),
            false,
            Some(Arc::new(|_s| Self::secondary_fixed_dim())),
            Some(Arc::new(|_s| Self::secondary_fixed())),
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }

    // -------------------------------------------------------------------------
    // Tertiary Fixed Roles
    // -------------------------------------------------------------------------

    pub fn tertiary_fixed() -> DynamicColor {
        DynamicColor::new(
            "tertiary_fixed",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::tertiary_fixed(),
                    Self::tertiary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn tertiary_fixed_dim() -> DynamicColor {
        DynamicColor::new(
            "tertiary_fixed_dim",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            true,
            Some(Arc::new(highest_surface)),
            None,
            Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)),
            Some(Arc::new(|_s| {
                ToneDeltaPair::new(
                    Self::tertiary_fixed(),
                    Self::tertiary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                )
            })),
        )
    }

    pub fn on_tertiary_fixed() -> DynamicColor {
        DynamicColor::new(
            "on_tertiary_fixed",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            false,
            Some(Arc::new(|_s| Self::tertiary_fixed_dim())),
            Some(Arc::new(|_s| Self::tertiary_fixed())),
            Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)),
            None,
        )
    }

    pub fn on_tertiary_fixed_variant() -> DynamicColor {
        DynamicColor::new(
            "on_tertiary_fixed_variant",
            Arc::new(|s| s.tertiary_palette),
            Arc::new(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            false,
            Some(Arc::new(|_s| Self::tertiary_fixed_dim())),
            Some(Arc::new(|_s| Self::tertiary_fixed())),
            Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)),
            None,
        )
    }
}
