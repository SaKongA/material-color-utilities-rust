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

//! Theme scoring system: Ranks color suggestions based on suitability for M3 themes.

use std::collections::BTreeMap;

use crate::cam::hct::Hct;
use crate::utils::{diff_degrees, sanitize_degrees_int, Argb};

const TARGET_CHROMA: f64 = 48.0;
const WEIGHT_PROPORTION: f64 = 0.7;
const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
const WEIGHT_CHROMA_BELOW: f64 = 0.1;
const CUTOFF_CHROMA: f64 = 5.0;
const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;

/// Options for scoring and ranking color suggestions.
#[derive(Debug, Clone)]
pub struct ScoreOptions {
    /// Number of desired colors to return.
    pub desired: usize,
    /// Fallback default color if no candidates are suitable (default: Google Blue `0xff4285f4`).
    pub fallback_color_argb: Argb,
    /// Controls whether to filter out low chroma or under-represented colors.
    pub filter: bool,
}

impl Default for ScoreOptions {
    fn default() -> Self {
        Self {
            desired: 4,
            fallback_color_argb: 0xff42_85f4,
            filter: true,
        }
    }
}

/// Ranks colors by suitability for Material Design 3 theming based on usage counts.
pub fn ranked_suggestions(
    argb_to_population: &BTreeMap<Argb, u32>,
    options: &ScoreOptions,
) -> Vec<Argb> {
    let mut colors_hct = Vec::new();
    let mut hue_population = [0u32; 360];
    let mut population_sum = 0.0f64;

    for (&argb, &population) in argb_to_population {
        let hct = Hct::from_int(argb);
        colors_hct.push(hct);
        let hue = hct.hue().floor() as usize;
        let hue = hue % 360;
        hue_population[hue] += population;
        population_sum += population as f64;
    }

    let mut hue_excited_proportions = [0.0f64; 360];
    for (hue, &count) in hue_population.iter().enumerate() {
        let proportion = (count as f64) / population_sum;
        for i in (hue as i32 - 14)..(hue as i32 + 16) {
            let neighbor_hue = sanitize_degrees_int(i) as usize;
            hue_excited_proportions[neighbor_hue] += proportion;
        }
    }

    let mut scored_hcts: Vec<(Hct, f64)> = Vec::new();
    for hct in colors_hct {
        let hue = sanitize_degrees_int(hct.hue().round() as i32) as usize;
        let proportion = hue_excited_proportions[hue];

        if options.filter
            && (hct.chroma() < CUTOFF_CHROMA || proportion <= CUTOFF_EXCITED_PROPORTION)
        {
            continue;
        }

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;
        let chroma_weight = if hct.chroma() < TARGET_CHROMA {
            WEIGHT_CHROMA_BELOW
        } else {
            WEIGHT_CHROMA_ABOVE
        };
        let chroma_score = (hct.chroma() - TARGET_CHROMA) * chroma_weight;
        let score = proportion_score + chroma_score;
        scored_hcts.push((hct, score));
    }

    scored_hcts.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut chosen_colors: Vec<Hct> = Vec::new();
    let mut difference_degrees = 90;
    while difference_degrees >= 15 {
        chosen_colors.clear();
        for &(hct, _) in &scored_hcts {
            let has_duplicate_hue = chosen_colors.iter().any(|&chosen_hct| {
                diff_degrees(hct.hue(), chosen_hct.hue()) < (difference_degrees as f64)
            });
            if !has_duplicate_hue {
                chosen_colors.push(hct);
                if chosen_colors.len() >= options.desired {
                    break;
                }
            }
        }
        if chosen_colors.len() >= options.desired {
            break;
        }
        difference_degrees -= 1;
    }

    let mut colors = Vec::new();
    if chosen_colors.is_empty() {
        colors.push(options.fallback_color_argb);
    }
    for chosen_hct in chosen_colors {
        colors.push(chosen_hct.to_int());
    }
    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prioritizes_chroma() {
        let mut argb_to_population = BTreeMap::new();
        argb_to_population.insert(0xff00_0000, 1);
        argb_to_population.insert(0xffff_ffff, 1);
        argb_to_population.insert(0xff00_00ff, 1);

        let ranked = ranked_suggestions(&argb_to_population, &ScoreOptions { desired: 4, ..Default::default() });
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff00_00ff);
    }

    #[test]
    fn test_prioritizes_chroma_when_proportions_equal() {
        let mut argb_to_population = BTreeMap::new();
        argb_to_population.insert(0xffff_0000, 1);
        argb_to_population.insert(0xff00_ff00, 1);
        argb_to_population.insert(0xff00_00ff, 1);

        let ranked = ranked_suggestions(&argb_to_population, &ScoreOptions { desired: 4, ..Default::default() });
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0], 0xffff_0000);
        assert_eq!(ranked[1], 0xff00_ff00);
        assert_eq!(ranked[2], 0xff00_00ff);
    }

    #[test]
    fn test_generates_gblue_when_no_colors_available() {
        let mut argb_to_population = BTreeMap::new();
        argb_to_population.insert(0xff00_0000, 1);

        let ranked = ranked_suggestions(&argb_to_population, &ScoreOptions { desired: 4, ..Default::default() });
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff42_85f4);
    }

    #[test]
    fn test_dedupes_nearby_hues() {
        let mut argb_to_population = BTreeMap::new();
        argb_to_population.insert(0xff00_8772, 1);
        argb_to_population.insert(0xff31_8477, 1);

        let ranked = ranked_suggestions(&argb_to_population, &ScoreOptions { desired: 4, ..Default::default() });
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff00_8772);
    }

    #[test]
    fn test_maximizes_hue_distance() {
        let mut argb_to_population = BTreeMap::new();
        argb_to_population.insert(0xff00_8772, 1);
        argb_to_population.insert(0xff00_8587, 1);
        argb_to_population.insert(0xff00_7ebc, 1);

        let ranked = ranked_suggestions(&argb_to_population, &ScoreOptions { desired: 2, ..Default::default() });
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0], 0xff00_7ebc);
        assert_eq!(ranked[1], 0xff00_8772);
    }

    #[test]
    fn test_generated_scenarios() {
        // Scenario 1
        let mut scenario1 = BTreeMap::new();
        scenario1.insert(0xff7e_a16d, 67);
        scenario1.insert(0xffd8_ccae, 67);
        scenario1.insert(0xff83_5c0d, 49);
        let ranked1 = ranked_suggestions(
            &scenario1,
            &ScoreOptions { desired: 3, fallback_color_argb: 0xff8d_3819, filter: false },
        );
        assert_eq!(ranked1.len(), 3);
        assert_eq!(ranked1[0], 0xff7e_a16d);
        assert_eq!(ranked1[1], 0xffd8_ccae);
        assert_eq!(ranked1[2], 0xff83_5c0d);

        // Scenario 2
        let mut scenario2 = BTreeMap::new();
        scenario2.insert(0xffd3_3881, 14);
        scenario2.insert(0xff32_05cc, 77);
        scenario2.insert(0xff0b_48cf, 36);
        scenario2.insert(0xffa0_8f5d, 81);
        let ranked2 = ranked_suggestions(
            &scenario2,
            &ScoreOptions { desired: 4, fallback_color_argb: 0xff7d_772b, filter: true },
        );
        assert_eq!(ranked2.len(), 3);
        assert_eq!(ranked2[0], 0xff32_05cc);
        assert_eq!(ranked2[1], 0xffa0_8f5d);
        assert_eq!(ranked2[2], 0xffd3_3881);
    }
}
