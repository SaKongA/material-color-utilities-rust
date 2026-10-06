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

//! Tone delta constraint specification between two dynamic colors.

use super::dynamic_color::DynamicColor;

/// Describes the relative tone relation between two colors in a delta pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TonePolarity {
    /// Role A should be darker than Role B.
    Darker,
    /// Role A should be lighter than Role B.
    Lighter,
    /// Role A should be nearer to the background/surface than Role B.
    Nearer,
    /// Role A should be farther from the background/surface than Role B.
    Farther,
}

/// Documents a constraint between two [`DynamicColor`]s, in which their tones must
/// have a minimum distance from each other.
///
/// Prefer a DynamicColor with a background; this is for special cases when
/// designers want tonal distance, literally contrast, between two colors that
/// don't have a background / foreground relationship or a contrast guarantee.
#[derive(Clone)]
pub struct ToneDeltaPair {
    /// The first role in the pair.
    pub role_a: DynamicColor,
    /// The second role in the pair.
    pub role_b: DynamicColor,
    /// Required difference between tones. Absolute value, positive number.
    pub delta: f64,
    /// The relative relation between tones of role_a and role_b.
    pub polarity: TonePolarity,
    /// Whether these two roles should stay on the same side of the "awkward zone" (T50-59).
    pub stay_together: bool,
}

impl ToneDeltaPair {
    /// Creates a tone delta constraint between two colors.
    pub fn new(
        role_a: DynamicColor,
        role_b: DynamicColor,
        delta: f64,
        polarity: TonePolarity,
        stay_together: bool,
    ) -> Self {
        Self {
            role_a,
            role_b,
            delta,
            polarity,
            stay_together,
        }
    }
}

impl std::fmt::Debug for ToneDeltaPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToneDeltaPair")
            .field("role_a", &self.role_a.name)
            .field("role_b", &self.role_b.name)
            .field("delta", &self.delta)
            .field("polarity", &self.polarity)
            .field("stay_together", &self.stay_together)
            .finish()
    }
}
