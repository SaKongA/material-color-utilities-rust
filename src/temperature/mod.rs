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

//! Color temperature theory, complementary, and analogous colors cache.

use std::collections::BTreeMap;

use crate::cam::hct::Hct;
use crate::quantize::lab::lab_from_int;
use crate::utils::{sanitize_degrees_double, sanitize_degrees_int, PI};

/// Design utilities using color temperature theory.
///
/// Computes analogous colors, complementary colors, and maintains a cache
/// to efficiently generate data when requested.
pub struct TemperatureCache {
    input: Hct,
    precomputed_complement: Option<Hct>,
    precomputed_hcts_by_temp: Option<Vec<Hct>>,
    precomputed_hcts_by_hue: Option<Vec<Hct>>,
    precomputed_temps_by_hct: Option<BTreeMap<Hct, f64>>,
}

impl TemperatureCache {
    /// Creates a new `TemperatureCache` for the given input color.
    pub fn new(input: Hct) -> Self {
        Self {
            input,
            precomputed_complement: None,
            precomputed_hcts_by_temp: None,
            precomputed_hcts_by_hue: None,
            precomputed_temps_by_hct: None,
        }
    }

    /// Evaluates the cool-warm factor of a color using Ou, Woodcock and Wright's algorithm.
    ///
    /// Values below 0.0 are considered cool; above 0.0 are considered warm.
    pub fn raw_temperature(color: Hct) -> f64 {
        let lab = lab_from_int(color.to_int());
        let hue = sanitize_degrees_double(lab.b.atan2(lab.a) * 180.0 / PI);
        let chroma = lab.a.hypot(lab.b);
        -0.5 + 0.02 * chroma.powf(1.07) * (sanitize_degrees_double(hue - 50.0) * PI / 180.0).cos()
    }

    /// Determines if an angle is between two other angles, rotating clockwise.
    fn is_between(angle: f64, a: f64, b: f64) -> bool {
        if a < b {
            a <= angle && angle <= b
        } else {
            a <= angle || angle <= b
        }
    }

    /// Returns all HCT colors across all 360 integer hues with the input's chroma and tone.
    pub fn get_hcts_by_hue(&mut self) -> &[Hct] {
        if self.precomputed_hcts_by_hue.is_none() {
            let mut hcts = Vec::with_capacity(361);
            let mut hue = 0.0;
            while hue <= 360.0 {
                hcts.push(Hct::new(hue, self.input.chroma(), self.input.tone()));
                hue += 1.0;
            }
            self.precomputed_hcts_by_hue = Some(hcts);
        }
        self.precomputed_hcts_by_hue.as_ref().unwrap()
    }

    /// Maps all evaluated HCTs to their raw temperatures.
    pub fn get_temps_by_hct(&mut self) -> &BTreeMap<Hct, f64> {
        if self.precomputed_temps_by_hct.is_none() {
            let mut all_hcts = self.get_hcts_by_hue().to_vec();
            all_hcts.push(self.input);

            let mut temperatures_by_hct = BTreeMap::new();
            for hct in all_hcts {
                temperatures_by_hct.insert(hct, Self::raw_temperature(hct));
            }
            self.precomputed_temps_by_hct = Some(temperatures_by_hct);
        }
        self.precomputed_temps_by_hct.as_ref().unwrap()
    }

    /// Returns all HCTs sorted from coldest first to warmest last.
    pub fn get_hcts_by_temp(&mut self) -> &[Hct] {
        if self.precomputed_hcts_by_temp.is_none() {
            let mut hcts = self.get_hcts_by_hue().to_vec();
            hcts.push(self.input);

            let temps = self.get_temps_by_hct().clone();
            hcts.sort_by(|a, b| {
                let temp_a = temps.get(a).copied().unwrap_or(0.0);
                let temp_b = temps.get(b).copied().unwrap_or(0.0);
                temp_a.total_cmp(&temp_b)
            });
            self.precomputed_hcts_by_temp = Some(hcts);
        }
        self.precomputed_hcts_by_temp.as_ref().unwrap()
    }

    /// Returns the coldest color with the same chroma and tone as the input.
    pub fn get_coldest(&mut self) -> Hct {
        self.get_hcts_by_temp()[0]
    }

    /// Returns the warmest color with the same chroma and tone as the input.
    pub fn get_warmest(&mut self) -> Hct {
        let last_idx = self.get_hcts_by_temp().len() - 1;
        self.get_hcts_by_temp()[last_idx]
    }

    /// Returns the relative temperature (in range `[0.0, 1.0]`) of an HCT color.
    pub fn get_relative_temperature(&mut self, hct: Hct) -> f64 {
        let coldest = self.get_coldest();
        let warmest = self.get_warmest();
        let coldest_temp = *self.get_temps_by_hct().get(&coldest).unwrap();
        let warmest_temp = *self.get_temps_by_hct().get(&warmest).unwrap();
        let range = warmest_temp - coldest_temp;
        let hct_temp = *self.get_temps_by_hct().get(&hct).unwrap();
        let difference_from_coldest = hct_temp - coldest_temp;

        if range == 0.0 {
            0.5
        } else {
            difference_from_coldest / range
        }
    }

    /// Returns a color that aesthetically complements the input color.
    pub fn complement(&mut self) -> Hct {
        self.get_complement()
    }

    /// Returns a color that aesthetically complements the input color.
    pub fn get_complement(&mut self) -> Hct {
        if let Some(complement) = self.precomputed_complement {
            return complement;
        }

        let coldest = self.get_coldest();
        let coldest_hue = coldest.hue();
        let coldest_temp = *self.get_temps_by_hct().get(&coldest).unwrap();

        let warmest = self.get_warmest();
        let warmest_hue = warmest.hue();
        let warmest_temp = *self.get_temps_by_hct().get(&warmest).unwrap();
        let range = warmest_temp - coldest_temp;

        let start_hue_is_coldest_to_warmest =
            Self::is_between(self.input.hue(), coldest_hue, warmest_hue);
        let start_hue = if start_hue_is_coldest_to_warmest {
            warmest_hue
        } else {
            coldest_hue
        };
        let end_hue = if start_hue_is_coldest_to_warmest {
            coldest_hue
        } else {
            warmest_hue
        };

        let direction_of_rotation = 1.0;
        let mut smallest_error = 1000.0;
        let input_hue_round = self.input.hue().round() as usize;
        let mut answer = self.get_hcts_by_hue()[input_hue_round];

        let complement_relative_temp = 1.0 - self.get_relative_temperature(self.input);

        let mut hue_addend = 0.0;
        while hue_addend <= 360.0 {
            let hue = sanitize_degrees_double(start_hue + direction_of_rotation * hue_addend);
            if Self::is_between(hue, start_hue, end_hue) {
                let possible_answer = self.get_hcts_by_hue()[hue.round() as usize];
                let possible_temp = *self.get_temps_by_hct().get(&possible_answer).unwrap();
                let relative_temp = (possible_temp - coldest_temp) / range;
                let error = (complement_relative_temp - relative_temp).abs();
                if error < smallest_error {
                    smallest_error = error;
                    answer = possible_answer;
                }
            }
            hue_addend += 1.0;
        }

        self.precomputed_complement = Some(answer);
        answer
    }

    /// Returns 5 colors that pair well with the input color (12 divisions).
    pub fn get_analogous_colors(&mut self) -> Vec<Hct> {
        self.get_analogous_colors_with_params(5, 12)
    }

    /// Returns `count` colors equidistant in temperature across `divisions` wheel sections.
    pub fn analogous_colors(&mut self, count: usize, divisions: usize) -> Vec<Hct> {
        self.get_analogous_colors_with_params(count, divisions)
    }

    /// Returns `count` colors equidistant in temperature across `divisions` wheel sections.
    pub fn get_analogous_colors_with_params(&mut self, count: usize, divisions: usize) -> Vec<Hct> {
        let start_hue = self.input.hue().round() as i32;
        let start_hct = self.get_hcts_by_hue()[start_hue as usize];
        let mut last_temp = self.get_relative_temperature(start_hct);

        let mut all_colors = Vec::new();
        all_colors.push(start_hct);

        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = sanitize_degrees_int(start_hue + i);
            let hct = self.get_hcts_by_hue()[hue as usize];
            let temp = self.get_relative_temperature(hct);
            let temp_delta = (temp - last_temp).abs();
            last_temp = temp;
            absolute_total_temp_delta += temp_delta;
        }

        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / (divisions as f64);
        let mut total_temp_delta = 0.0;
        last_temp = self.get_relative_temperature(start_hct);

        while all_colors.len() < divisions {
            let hue = sanitize_degrees_int(start_hue + hue_addend);
            let hct = self.get_hcts_by_hue()[hue as usize];
            let temp = self.get_relative_temperature(hct);
            let temp_delta = (temp - last_temp).abs();
            total_temp_delta += temp_delta;

            let mut desired_total_temp_delta_for_index = (all_colors.len() as f64) * temp_step;
            let mut index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
            let mut index_addend = 1;

            while index_satisfied && all_colors.len() < divisions {
                all_colors.push(hct);
                desired_total_temp_delta_for_index =
                    ((all_colors.len() + index_addend) as f64) * temp_step;
                index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
                index_addend += 1;
            }

            last_temp = temp;
            hue_addend += 1;

            if hue_addend > 360 {
                while all_colors.len() < divisions {
                    all_colors.push(hct);
                }
                break;
            }
        }

        let mut answers = Vec::new();
        answers.push(self.input);

        let ccw_count = ((count as f64 - 1.0) / 2.0).floor() as usize;
        for i in 1..=ccw_count {
            let mut index = -(i as isize);
            while index < 0 {
                index += all_colors.len() as isize;
            }
            let index = (index as usize) % all_colors.len();
            answers.insert(0, all_colors[index]);
        }

        let cw_count = count - ccw_count - 1;
        for i in 1..=cw_count {
            let index = i % all_colors.len();
            answers.push(all_colors[index]);
        }

        answers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Argb;

    fn double_near(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn test_raw_temperature() {
        let blue_hct = Hct::from_int(0xff00_00ff);
        assert!(double_near(
            TemperatureCache::raw_temperature(blue_hct),
            -1.393,
            0.001
        ));

        let red_hct = Hct::from_int(0xffff_0000);
        assert!(double_near(
            TemperatureCache::raw_temperature(red_hct),
            2.351,
            0.001
        ));

        let green_hct = Hct::from_int(0xff00_ff00);
        assert!(double_near(
            TemperatureCache::raw_temperature(green_hct),
            -0.267,
            0.001
        ));

        let white_hct = Hct::from_int(0xffff_ffff);
        assert!(double_near(
            TemperatureCache::raw_temperature(white_hct),
            -0.5,
            0.001
        ));

        let black_hct = Hct::from_int(0xff00_0000);
        assert!(double_near(
            TemperatureCache::raw_temperature(black_hct),
            -0.5,
            0.001
        ));
    }

    #[test]
    fn test_complements() {
        let mut blue_cache = TemperatureCache::new(Hct::from_int(0xff00_00ff));
        assert_eq!(blue_cache.get_complement().to_int(), 0xff9d_0002 as Argb);

        let mut red_cache = TemperatureCache::new(Hct::from_int(0xffff_0000));
        assert_eq!(red_cache.get_complement().to_int(), 0xff00_7bfc as Argb);

        let mut green_cache = TemperatureCache::new(Hct::from_int(0xff00_ff00));
        assert_eq!(green_cache.get_complement().to_int(), 0xffff_d2c9 as Argb);

        let mut white_cache = TemperatureCache::new(Hct::from_int(0xffff_ffff));
        assert_eq!(white_cache.get_complement().to_int(), 0xffff_ffff as Argb);

        let mut black_cache = TemperatureCache::new(Hct::from_int(0xff00_0000));
        assert_eq!(black_cache.get_complement().to_int(), 0xff00_0000 as Argb);
    }

    #[test]
    fn test_analogous_blue() {
        let mut blue_cache = TemperatureCache::new(Hct::from_int(0xff00_00ff));
        let blue_analogous = blue_cache.get_analogous_colors();
        assert_eq!(blue_analogous[0].to_int(), 0xff00_590c as Argb);
        assert_eq!(blue_analogous[1].to_int(), 0xff00_564e as Argb);
        assert_eq!(blue_analogous[2].to_int(), 0xff00_00ff as Argb);
        assert_eq!(blue_analogous[3].to_int(), 0xff67_00cc as Argb);
        assert_eq!(blue_analogous[4].to_int(), 0xff81_009f as Argb);
    }

    #[test]
    fn test_analogous_red() {
        let mut red_cache = TemperatureCache::new(Hct::from_int(0xffff_0000));
        let red_analogous = red_cache.get_analogous_colors();
        assert_eq!(red_analogous[0].to_int(), 0xfff6_0082 as Argb);
        assert_eq!(red_analogous[1].to_int(), 0xfffc_004c as Argb);
        assert_eq!(red_analogous[2].to_int(), 0xffff_0000 as Argb);
        assert_eq!(red_analogous[3].to_int(), 0xffd9_5500 as Argb);
        assert_eq!(red_analogous[4].to_int(), 0xffaf_7200 as Argb);
    }

    #[test]
    fn test_analogous_green() {
        let mut green_cache = TemperatureCache::new(Hct::from_int(0xff00_ff00));
        let green_analogous = green_cache.get_analogous_colors();
        assert_eq!(green_analogous[0].to_int(), 0xffce_e900 as Argb);
        assert_eq!(green_analogous[1].to_int(), 0xff92_f500 as Argb);
        assert_eq!(green_analogous[2].to_int(), 0xff00_ff00 as Argb);
        assert_eq!(green_analogous[3].to_int(), 0xff00_fd6f as Argb);
        assert_eq!(green_analogous[4].to_int(), 0xff00_fab3 as Argb);
    }

    #[test]
    fn test_analogous_black_white() {
        let mut black_cache = TemperatureCache::new(Hct::from_int(0xff00_0000));
        for color in black_cache.get_analogous_colors() {
            assert_eq!(color.to_int(), 0xff00_0000 as Argb);
        }

        let mut white_cache = TemperatureCache::new(Hct::from_int(0xffff_ffff));
        for color in white_cache.get_analogous_colors() {
            assert_eq!(color.to_int(), 0xffff_ffff as Argb);
        }
    }
}
