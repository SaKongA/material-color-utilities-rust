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

//! Weighted square-error k-means color quantizer (WSMeans).

use std::collections::{BTreeMap, HashMap};

use crate::quantize::lab::{int_from_lab, lab_from_int, Lab};
use crate::utils::Argb;

const MAX_ITERATIONS: usize = 100;
const MIN_DELTA_E: f64 = 3.0;

/// Result of quantization containing color population counts and input pixel mappings.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct QuantizerResult {
    pub color_to_count: BTreeMap<Argb, u32>,
    pub input_pixel_to_cluster_pixel: BTreeMap<Argb, Argb>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DistanceToIndex {
    distance: f64,
    index: usize,
}

impl Eq for DistanceToIndex {}

impl PartialOrd for DistanceToIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DistanceToIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Swatch {
    argb: Argb,
    population: usize,
}

impl Ord for Swatch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.population.cmp(&self.population)
    }
}

impl PartialOrd for Swatch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A simple linear congruential pseudo-random generator matching C++ `rand()` seeded with 42688.
struct CRand {
    state: u32,
}

impl CRand {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(214013).wrapping_add(2531011);
        (self.state >> 16) & 0x7fff
    }

    fn next_f64(&mut self) -> f64 {
        self.next() as f64 / 32767.0
    }
}

/// Quantizes `input_pixels` using weighted square-error k-means in Lab color space.
pub fn quantize_wsmeans(
    input_pixels: &[Argb],
    starting_clusters: &[Argb],
    mut max_colors: u16,
) -> QuantizerResult {
    if max_colors == 0 || input_pixels.is_empty() {
        return QuantizerResult::default();
    }
    if max_colors > 256 {
        max_colors = 256;
    }
    let max_colors = max_colors as usize;

    let mut pixel_to_count = HashMap::new();
    let mut pixels = Vec::with_capacity(input_pixels.len());
    let mut points = Vec::with_capacity(input_pixels.len());

    for &pixel in input_pixels {
        match pixel_to_count.get_mut(&pixel) {
            Some(count) => {
                *count += 1;
            }
            None => {
                pixels.push(pixel);
                points.push(lab_from_int(pixel));
                pixel_to_count.insert(pixel, 1);
            }
        }
    }

    let mut cluster_count = max_colors.min(points.len());
    if !starting_clusters.is_empty() {
        cluster_count = cluster_count.min(starting_clusters.len());
    }

    let mut clusters: Vec<Lab> = starting_clusters
        .iter()
        .map(|&argb| lab_from_int(argb))
        .collect();

    let mut rng = CRand::new(42688);
    let additional_clusters_needed = cluster_count.saturating_sub(clusters.len());
    if starting_clusters.is_empty() && additional_clusters_needed > 0 {
        for _ in 0..additional_clusters_needed {
            let l = rng.next_f64() * 100.0;
            let a = rng.next_f64() * 200.0 - 100.0;
            let b = rng.next_f64() * 200.0 - 100.0;
            clusters.push(Lab::new(l, a, b));
        }
    }

    let mut cluster_indices = Vec::with_capacity(points.len());
    let mut rng_indices = CRand::new(42688);
    for _ in 0..points.len() {
        cluster_indices.push((rng_indices.next() as usize) % cluster_count);
    }

    let mut distance_to_index_matrix = vec![
        vec![
            DistanceToIndex {
                distance: 0.0,
                index: 0
            };
            cluster_count
        ];
        cluster_count
    ];

    let mut pixel_count_sums = [0usize; 256];

    for iteration in 0..MAX_ITERATIONS {
        for i in 0..cluster_count {
            distance_to_index_matrix[i][i].distance = 0.0;
            distance_to_index_matrix[i][i].index = i;
            for j in (i + 1)..cluster_count {
                let distance = clusters[i].delta_e(&clusters[j]);
                distance_to_index_matrix[j][i].distance = distance;
                distance_to_index_matrix[j][i].index = i;
                distance_to_index_matrix[i][j].distance = distance;
                distance_to_index_matrix[i][j].index = j;
            }
            distance_to_index_matrix[i].sort();
        }

        let mut color_moved = false;
        for i in 0..points.len() {
            let point = points[i];
            let previous_cluster_index = cluster_indices[i];
            let previous_cluster = clusters[previous_cluster_index];
            let previous_distance = point.delta_e(&previous_cluster);
            let mut minimum_distance = previous_distance;
            let mut new_cluster_index = None;

            for j in 0..cluster_count {
                if distance_to_index_matrix[previous_cluster_index][j].distance
                    >= 4.0 * previous_distance
                {
                    continue;
                }
                let distance = point.delta_e(&clusters[j]);
                if distance < minimum_distance {
                    minimum_distance = distance;
                    new_cluster_index = Some(j);
                }
            }

            if let Some(new_idx) = new_cluster_index {
                let distance_change = (minimum_distance.sqrt() - previous_distance.sqrt()).abs();
                if distance_change > MIN_DELTA_E {
                    color_moved = true;
                    cluster_indices[i] = new_idx;
                }
            }
        }

        if !color_moved && iteration != 0 {
            break;
        }

        let mut component_a_sums = [0.0f64; 256];
        let mut component_b_sums = [0.0f64; 256];
        let mut component_c_sums = [0.0f64; 256];
        for item in pixel_count_sums.iter_mut().take(cluster_count) {
            *item = 0;
        }

        for i in 0..points.len() {
            let cluster_idx = cluster_indices[i];
            let point = points[i];
            let count = pixel_to_count[&pixels[i]];

            pixel_count_sums[cluster_idx] += count;
            component_a_sums[cluster_idx] += point.l * (count as f64);
            component_b_sums[cluster_idx] += point.a * (count as f64);
            component_c_sums[cluster_idx] += point.b * (count as f64);
        }

        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                clusters[i] = Lab::new(0.0, 0.0, 0.0);
                continue;
            }
            let a = component_a_sums[i] / (count as f64);
            let b = component_b_sums[i] / (count as f64);
            let c = component_c_sums[i] / (count as f64);
            clusters[i] = Lab::new(a, b, c);
        }
    }

    let mut swatches: Vec<Swatch> = Vec::new();
    let mut all_cluster_argbs = Vec::new();

    for i in 0..cluster_count {
        let possible_new_cluster = int_from_lab(clusters[i]);
        all_cluster_argbs.push(possible_new_cluster);

        let count = pixel_count_sums[i];
        if count == 0 {
            continue;
        }

        let mut use_new_cluster = true;
        for swatch in &mut swatches {
            if swatch.argb == possible_new_cluster {
                swatch.population += count;
                use_new_cluster = false;
                break;
            }
        }

        if use_new_cluster {
            swatches.push(Swatch {
                argb: possible_new_cluster,
                population: count,
            });
        }
    }
    swatches.sort();

    let mut color_to_count = BTreeMap::new();
    for swatch in swatches {
        color_to_count.insert(swatch.argb, swatch.population as u32);
    }

    let mut input_pixel_to_cluster_pixel = BTreeMap::new();
    for i in 0..points.len() {
        let pixel = pixels[i];
        let cluster_index = cluster_indices[i];
        let cluster_argb = all_cluster_argbs[cluster_index];
        input_pixel_to_cluster_pixel.insert(pixel, cluster_argb);
    }

    QuantizerResult {
        color_to_count,
        input_pixel_to_cluster_pixel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsmeans_single_red() {
        let pixels = vec![0xffff_0000];
        let starting = vec![];
        let result = quantize_wsmeans(&pixels, &starting, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xffff_0000), Some(&1));
    }

    #[test]
    fn test_wsmeans_five_blue() {
        let pixels = vec![0xff00_00ff; 5];
        let starting = vec![];
        let result = quantize_wsmeans(&pixels, &starting, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xff00_00ff), Some(&5));
    }
}
