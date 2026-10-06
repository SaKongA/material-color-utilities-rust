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

//! Celebi quantizer: Combines Wu and WSMeans algorithms for robust color extraction.

use crate::quantize::wsmeans::{quantize_wsmeans, QuantizerResult};
use crate::quantize::wu::quantize_wu;
use crate::utils::{is_opaque, Argb};

/// Quantizes an image by first finding cluster initializations with Wu,
/// followed by weighted square-error k-means in Lab color space.
pub fn quantize_celebi(pixels: &[Argb], mut max_colors: u16) -> QuantizerResult {
    if max_colors == 0 || pixels.is_empty() {
        return QuantizerResult::default();
    }
    if max_colors > 256 {
        max_colors = 256;
    }

    let opaque_pixels: Vec<Argb> = pixels
        .iter()
        .copied()
        .filter(|&pixel| is_opaque(pixel))
        .collect();

    if opaque_pixels.is_empty() {
        return QuantizerResult::default();
    }

    let wu_result = quantize_wu(&opaque_pixels, max_colors);
    quantize_wsmeans(&opaque_pixels, &wu_result, max_colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celebi_one_red() {
        let pixels = vec![0xffff_0000];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xffff_0000), Some(&1));
    }

    #[test]
    fn test_celebi_one_green() {
        let pixels = vec![0xff00_ff00];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xff00_ff00), Some(&1));
    }

    #[test]
    fn test_celebi_one_blue() {
        let pixels = vec![0xff00_00ff];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xff00_00ff), Some(&1));
    }

    #[test]
    fn test_celebi_five_blue() {
        let pixels = vec![0xff00_00ff; 5];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 1);
        assert_eq!(result.color_to_count.get(&0xff00_00ff), Some(&5));
    }

    #[test]
    fn test_celebi_rgb() {
        let pixels = vec![0xffff_0000, 0xff00_ff00, 0xff00_00ff];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 3);
        assert_eq!(result.color_to_count.get(&0xffff_0000), Some(&1));
        assert_eq!(result.color_to_count.get(&0xff00_ff00), Some(&1));
        assert_eq!(result.color_to_count.get(&0xff00_00ff), Some(&1));
    }

    #[test]
    fn test_celebi_two_red_three_green() {
        let pixels = vec![
            0xffff_0000,
            0xffff_0000,
            0xff00_ff00,
            0xff00_ff00,
            0xff00_ff00,
        ];
        let result = quantize_celebi(&pixels, 256);
        assert_eq!(result.color_to_count.len(), 2);
        assert_eq!(result.color_to_count.get(&0xffff_0000), Some(&2));
        assert_eq!(result.color_to_count.get(&0xff00_ff00), Some(&3));
    }

    #[test]
    fn test_celebi_transparent_and_empty() {
        let pixels = vec![0xffffffff];
        let result = quantize_celebi(&pixels, 0);
        assert!(result.color_to_count.is_empty());

        let transparent_pixels = vec![0x20f9_3013];
        let result2 = quantize_celebi(&transparent_pixels, 1);
        assert!(result2.color_to_count.is_empty());
    }
}
