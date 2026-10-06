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

//! Dynamic color calculation and constraint resolution engine.

use std::sync::Arc;

use crate::cam::Hct;
use crate::contrast::{darker, darker_unsafe, lighter, lighter_unsafe, ratio_of_tones};
use crate::palettes::TonalPalette;
use crate::utils::Argb;

use super::contrast_curve::ContrastCurve;
use super::dynamic_scheme::DynamicScheme;
use super::tone_delta_pair::{ToneDeltaPair, TonePolarity};

/// Function signature for computing a tonal palette from a dynamic scheme.
pub type PaletteFn = Arc<dyn Fn(&DynamicScheme) -> TonalPalette + Send + Sync>;
/// Function signature for computing a tone from a dynamic scheme.
pub type ToneFn = Arc<dyn Fn(&DynamicScheme) -> f64 + Send + Sync>;
/// Function signature for computing a dynamic color from a dynamic scheme.
pub type DynamicColorFn = Arc<dyn Fn(&DynamicScheme) -> DynamicColor + Send + Sync>;
/// Function signature for computing a tone delta constraint from a dynamic scheme.
pub type ToneDeltaPairFn = Arc<dyn Fn(&DynamicScheme) -> ToneDeltaPair + Send + Sync>;

/// Given a background tone, find a foreground tone, while ensuring they reach
/// a contrast ratio that is as close to `ratio` as possible.
///
/// # Arguments
/// * `bg_tone` - Tone in HCT. Range is 0 to 100.
/// * `ratio` - The contrast ratio desired between `bg_tone` and the return value.
pub fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = lighter_unsafe(bg_tone, ratio);
    let darker_tone = darker_unsafe(bg_tone, ratio);
    let lighter_ratio = ratio_of_tones(lighter_tone, bg_tone);
    let darker_ratio = ratio_of_tones(darker_tone, bg_tone);
    let prefer_lighter = tone_prefers_light_foreground(bg_tone);

    if prefer_lighter {
        let negligible_difference =
            (lighter_ratio - darker_ratio).abs() < 0.1 && lighter_ratio < ratio && darker_ratio < ratio;
        if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible_difference {
            lighter_tone
        } else {
            darker_tone
        }
    } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
        darker_tone
    } else {
        lighter_tone
    }
}

/// Adjust a tone such that white has 4.5 contrast, if the tone is
/// reasonably close to supporting it.
pub fn enable_light_foreground(tone: f64) -> f64 {
    if tone_prefers_light_foreground(tone) && !tone_allows_light_foreground(tone) {
        49.0
    } else {
        tone
    }
}

/// Returns whether `tone` prefers a light foreground.
///
/// People prefer white foregrounds on ~T60-70. Observed over time, and also
/// by Andrew Somers during research for APCA.
///
/// T60 used to create the smallest discontinuity possible when skipping
/// down to T49 in order to ensure light foregrounds.
pub fn tone_prefers_light_foreground(tone: f64) -> bool {
    tone.round() < 60.0
}

/// Returns whether `tone` can reach a contrast ratio of 4.5 with a lighter color.
pub fn tone_allows_light_foreground(tone: f64) -> bool {
    tone.round() <= 49.0
}

/// A color that adapts dynamically based on UI environment, contrast level,
/// dark mode setting, and relationships with other colors.
#[derive(Clone)]
pub struct DynamicColor {
    /// The name of the dynamic color role.
    pub name: String,
    /// Function that provides a `TonalPalette` given a `DynamicScheme`.
    pub palette: PaletteFn,
    /// Function that provides a base tone given a `DynamicScheme`.
    pub tone: ToneFn,
    /// Whether this dynamic color is a background role.
    pub is_background: bool,
    /// The background role of the dynamic color, if it exists.
    pub background: Option<DynamicColorFn>,
    /// A second background role of the dynamic color, if it exists.
    pub second_background: Option<DynamicColorFn>,
    /// Contrast curve specifying how contrast against background behaves at different contrast levels.
    pub contrast_curve: Option<ContrastCurve>,
    /// A tone delta constraint between two colors.
    pub tone_delta_pair: Option<ToneDeltaPairFn>,
}

impl std::fmt::Debug for DynamicColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicColor")
            .field("name", &self.name)
            .field("is_background", &self.is_background)
            .field("contrast_curve", &self.contrast_curve)
            .finish()
    }
}

impl DynamicColor {
    /// Full constructor for `DynamicColor`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        palette: PaletteFn,
        tone: ToneFn,
        is_background: bool,
        background: Option<DynamicColorFn>,
        second_background: Option<DynamicColorFn>,
        contrast_curve: Option<ContrastCurve>,
        tone_delta_pair: Option<ToneDeltaPairFn>,
    ) -> Self {
        Self {
            name: name.into(),
            palette,
            tone,
            is_background,
            background,
            second_background,
            contrast_curve,
            tone_delta_pair,
        }
    }

    /// Convenience constructor requiring only name, palette, and tone.
    pub fn from_palette(
        name: impl Into<String>,
        palette: PaletteFn,
        tone: ToneFn,
    ) -> Self {
        Self {
            name: name.into(),
            palette,
            tone,
            is_background: false,
            background: None,
            second_background: None,
            contrast_curve: None,
            tone_delta_pair: None,
        }
    }

    /// Returns the 32-bit ARGB representation of this dynamic color for the given scheme.
    pub fn get_argb(&self, scheme: &DynamicScheme) -> Argb {
        (self.palette)(scheme).get(self.get_tone(scheme))
    }

    /// Returns the HCT representation of this dynamic color for the given scheme.
    pub fn get_hct(&self, scheme: &DynamicScheme) -> Hct {
        Hct::from_int(self.get_argb(scheme))
    }

    /// Resolves and returns the tone (0.0 to 100.0) of this color in the given scheme,
    /// applying contrast adjustments, delta constraints, and awkward-zone avoidance.
    pub fn get_tone(&self, scheme: &DynamicScheme) -> f64 {
        let decreasing_contrast = scheme.contrast_level < 0.0;

        // Case 1: dual foreground, pair of colors with delta constraint.
        if let Some(ref tone_delta_pair_fn) = self.tone_delta_pair {
            let tone_delta_pair = tone_delta_pair_fn(scheme);
            let role_a = tone_delta_pair.role_a;
            let role_b = tone_delta_pair.role_b;
            let delta = tone_delta_pair.delta;
            let polarity = tone_delta_pair.polarity;
            let stay_together = tone_delta_pair.stay_together;

            let bg = self
                .background
                .as_ref()
                .expect("Tone delta pair requires a background")(scheme);
            let bg_tone = bg.get_tone(scheme);

            let a_is_nearer = polarity == TonePolarity::Nearer
                || (polarity == TonePolarity::Lighter && !scheme.is_dark)
                || (polarity == TonePolarity::Darker && scheme.is_dark);
            let nearer = if a_is_nearer { &role_a } else { &role_b };
            let farther = if a_is_nearer { &role_b } else { &role_a };
            let am_nearer = self.name == nearer.name;
            let expansion_dir = if scheme.is_dark { 1.0 } else { -1.0 };

            // 1st round: solve to min, each
            let n_contrast = nearer
                .contrast_curve
                .expect("Nearer role requires contrast curve")
                .get(scheme.contrast_level);
            let f_contrast = farther
                .contrast_curve
                .expect("Farther role requires contrast curve")
                .get(scheme.contrast_level);

            // If a color is good enough, it is not adjusted.
            let n_initial_tone = (nearer.tone)(scheme);
            let mut n_tone = if ratio_of_tones(bg_tone, n_initial_tone) >= n_contrast {
                n_initial_tone
            } else {
                foreground_tone(bg_tone, n_contrast)
            };

            let f_initial_tone = (farther.tone)(scheme);
            let mut f_tone = if ratio_of_tones(bg_tone, f_initial_tone) >= f_contrast {
                f_initial_tone
            } else {
                foreground_tone(bg_tone, f_contrast)
            };

            if decreasing_contrast {
                // If decreasing contrast, adjust color to the "bare minimum" that satisfies contrast.
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }

            if (f_tone - n_tone) * expansion_dir >= delta {
                // Good! Tones satisfy the constraint; no change needed.
            } else {
                // 2nd round: expand farther to match delta.
                f_tone = (n_tone + delta * expansion_dir).clamp(0.0, 100.0);
                if (f_tone - n_tone) * expansion_dir >= delta {
                    // Good! Tones now satisfy the constraint; no change needed.
                } else {
                    // 3rd round: contract nearer to match delta.
                    n_tone = (f_tone - delta * expansion_dir).clamp(0.0, 100.0);
                }
            }

            // Avoids the 50-59 awkward zone.
            if (50.0..60.0).contains(&n_tone) {
                // If `nearer` is in the awkward zone, move it away, together with `farther`.
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = f_tone.max(n_tone + delta * expansion_dir);
                } else {
                    n_tone = 49.0;
                    f_tone = f_tone.min(n_tone + delta * expansion_dir);
                }
            } else if (50.0..60.0).contains(&f_tone) {
                if stay_together {
                    // Fixes both, to avoid two colors on opposite sides of the "awkward zone".
                    if expansion_dir > 0.0 {
                        n_tone = 60.0;
                        f_tone = f_tone.max(n_tone + delta * expansion_dir);
                    } else {
                        n_tone = 49.0;
                        f_tone = f_tone.min(n_tone + delta * expansion_dir);
                    }
                } else if expansion_dir > 0.0 {
                    f_tone = 60.0;
                } else {
                    f_tone = 49.0;
                }
            }

            // Returns `n_tone` if this color is `nearer`, otherwise `f_tone`.
            if am_nearer {
                n_tone
            } else {
                f_tone
            }
        } else {
            // Case 2: No contrast pair; just solve for itself.
            let mut answer = (self.tone)(scheme);

            let bg_fn = match self.background.as_ref() {
                Some(bg) => bg,
                None => return answer, // No adjustment for colors with no background.
            };

            let bg_tone = bg_fn(scheme).get_tone(scheme);
            let desired_ratio = self
                .contrast_curve
                .expect("Color with background requires contrast curve")
                .get(scheme.contrast_level);

            if ratio_of_tones(bg_tone, answer) >= desired_ratio {
                // Don't "improve" what's good enough.
            } else {
                // Rough improvement.
                answer = foreground_tone(bg_tone, desired_ratio);
            }

            if decreasing_contrast {
                answer = foreground_tone(bg_tone, desired_ratio);
            }

            if self.is_background && (50.0..60.0).contains(&answer) {
                // Must adjust
                if ratio_of_tones(49.0, bg_tone) >= desired_ratio {
                    answer = 49.0;
                } else {
                    answer = 60.0;
                }
            }

            if let Some(ref second_bg_fn) = self.second_background {
                // Case 3: Adjust for dual backgrounds.
                let bg_tone_1 = bg_fn(scheme).get_tone(scheme);
                let bg_tone_2 = second_bg_fn(scheme).get_tone(scheme);

                let upper = bg_tone_1.max(bg_tone_2);
                let lower = bg_tone_1.min(bg_tone_2);

                if ratio_of_tones(upper, answer) >= desired_ratio
                    && ratio_of_tones(lower, answer) >= desired_ratio
                {
                    return answer;
                }

                // The darkest light tone that satisfies the desired ratio,
                // or -1 if such ratio cannot be reached.
                let light_option = lighter(upper, desired_ratio);

                // The lightest dark tone that satisfies the desired ratio,
                // or -1 if such ratio cannot be reached.
                let dark_option = darker(lower, desired_ratio);

                // Tones suitable for the foreground.
                let mut availables = Vec::new();
                if light_option != -1.0 {
                    availables.push(light_option);
                }
                if dark_option != -1.0 {
                    availables.push(dark_option);
                }

                let prefers_light = tone_prefers_light_foreground(bg_tone_1)
                    || tone_prefers_light_foreground(bg_tone_2);
                if prefers_light {
                    return if light_option < 0.0 { 100.0 } else { light_option };
                }
                if availables.len() == 1 {
                    return availables[0];
                }
                return if dark_option < 0.0 { 0.0 } else { dark_option };
            }

            answer
        }
    }
}
