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

//! Disliked color detection and correction.
//!
//! Color science studies of color preference indicate universal distaste for
//! dark yellow-greens, correlated with distaste for biological waste and rotting food.

use crate::cam::hct::Hct;

/// Returns whether a color is considered universally disliked (dark non-neutral yellow-green).
pub fn is_disliked(hct: Hct) -> bool {
    let rounded_hue = hct.hue().round();
    let hue_passes = (90.0..=111.0).contains(&rounded_hue);
    let chroma_passes = hct.chroma().round() > 16.0;
    let tone_passes = hct.tone().round() < 65.0;

    hue_passes && chroma_passes && tone_passes
}

/// If a color is disliked, lightens its tone to make it aesthetically likable.
pub fn fix_if_disliked(hct: Hct) -> Hct {
    if is_disliked(hct) {
        Hct::new(hct.hue(), hct.chroma(), 70.0)
    } else {
        hct
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monk_skin_tones_liked() {
        let monk_skin_tones = [
            0xfff6_ede4, 0xfff3_e7db, 0xfff7_ead0, 0xffea_daba, 0xffd7_bd96,
            0xffa0_7e56, 0xff82_5c43, 0xff60_4134, 0xff3a_312a, 0xff29_2420,
        ];
        for &argb in &monk_skin_tones {
            assert!(!is_disliked(Hct::from_int(argb)));
        }
    }

    #[test]
    fn test_bile_colors_disliked_and_fixed() {
        let bile_colors = [
            0xff95_884b, 0xff71_6b40, 0xffb0_8e00, 0xff4c_4308, 0xff46_4521,
        ];
        for &argb in &bile_colors {
            let bile = Hct::from_int(argb);
            assert!(is_disliked(bile));
            let fixed = fix_if_disliked(bile);
            assert!(!is_disliked(fixed));
        }
    }

    #[test]
    fn test_tone_67_liked() {
        let color = Hct::new(100.0, 50.0, 67.0);
        assert!(!is_disliked(color));
        assert_eq!(fix_if_disliked(color).to_int(), color.to_int());
    }
}
