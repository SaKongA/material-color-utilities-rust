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

//! Contrast and accessibility calculations.

use crate::utils::{lstar_from_y, y_from_lstar};

/// Epsilon threshold for contrast ratio calculation differences.
const CONTRAST_RATIO_EPSILON: f64 = 0.04;

/// Generous tolerance delta to ensure gamut mapped colors meet target contrast.
const LUMINANCE_GAMUT_MAP_TOLERANCE: f64 = 0.4;

/// Computes the WCAG contrast ratio between two relative luminances (CIE Y).
pub fn ratio_of_ys(y1: f64, y2: f64) -> f64 {
    let lighter = if y1 > y2 { y1 } else { y2 };
    let darker = if lighter == y2 { y1 } else { y2 };
    (lighter + 5.0) / (darker + 5.0)
}

/// Computes the contrast ratio between two tones (in range `[0.0, 100.0]`).
pub fn ratio_of_tones(tone_a: f64, tone_b: f64) -> f64 {
    let tone_a = tone_a.clamp(0.0, 100.0);
    let tone_b = tone_b.clamp(0.0, 100.0);
    ratio_of_ys(y_from_lstar(tone_a), y_from_lstar(tone_b))
}

/// Returns a tone $\ge$ `tone` that ensures `ratio`, or `-1.0` if impossible.
pub fn lighter(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }

    let dark_y = y_from_lstar(tone);
    let light_y = ratio * (dark_y + 5.0) - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > CONTRAST_RATIO_EPSILON {
        return -1.0;
    }

    let value = lstar_from_y(light_y) + LUMINANCE_GAMUT_MAP_TOLERANCE;
    if !(0.0..=100.0).contains(&value) {
        return -1.0;
    }
    value
}

/// Returns a tone $\le$ `tone` that ensures `ratio`, or `-1.0` if impossible.
pub fn darker(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }

    let light_y = y_from_lstar(tone);
    let dark_y = ((light_y + 5.0) / ratio) - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);

    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > CONTRAST_RATIO_EPSILON {
        return -1.0;
    }

    let value = lstar_from_y(dark_y) - LUMINANCE_GAMUT_MAP_TOLERANCE;
    if !(0.0..=100.0).contains(&value) {
        return -1.0;
    }
    value
}

/// Returns a tone $\ge$ `tone` that ensures `ratio`, or `100.0` if impossible.
pub fn lighter_unsafe(tone: f64, ratio: f64) -> f64 {
    let lighter_safe = lighter(tone, ratio);
    if lighter_safe < 0.0 {
        100.0
    } else {
        lighter_safe
    }
}

/// Returns a tone $\le$ `tone` that ensures `ratio`, or `0.0` if impossible.
pub fn darker_unsafe(tone: f64, ratio: f64) -> f64 {
    let darker_safe = darker(tone, ratio);
    if darker_safe < 0.0 {
        0.0
    } else {
        darker_safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_ratio_of_tones_out_of_bounds_input() {
        assert!(double_near(ratio_of_tones(-10.0, 110.0), 21.0, 0.001));
    }

    #[test]
    fn test_lighter_impossible_ratio_errors() {
        assert!(double_near(lighter(90.0, 10.0), -1.0, 0.001));
    }

    #[test]
    fn test_lighter_out_of_bounds_input_above_errors() {
        assert!(double_near(lighter(110.0, 2.0), -1.0, 0.001));
    }

    #[test]
    fn test_lighter_out_of_bounds_input_below_errors() {
        assert!(double_near(lighter(-10.0, 2.0), -1.0, 0.001));
    }

    #[test]
    fn test_lighter_unsafe_returns_max_tone() {
        assert!(double_near(lighter_unsafe(100.0, 2.0), 100.0, 0.001));
    }

    #[test]
    fn test_darker_impossible_ratio_errors() {
        assert!(double_near(darker(10.0, 20.0), -1.0, 0.001));
    }

    #[test]
    fn test_darker_out_of_bounds_input_above_errors() {
        assert!(double_near(darker(110.0, 2.0), -1.0, 0.001));
    }

    #[test]
    fn test_darker_out_of_bounds_input_below_errors() {
        assert!(double_near(darker(-10.0, 2.0), -1.0, 0.001));
    }

    #[test]
    fn test_darker_unsafe_returns_min_tone() {
        assert!(double_near(darker_unsafe(0.0, 2.0), 0.0, 0.001));
    }
}
