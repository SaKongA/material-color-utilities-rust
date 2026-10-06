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

//! Wu's color quantizer algorithm.
//!
//! An efficient color quantization algorithm by Xiaolin Wu using box cutting
//! based on variance minimization.

use crate::utils::{argb_from_rgb, blue_from_argb, green_from_argb, red_from_argb, Argb};

#[derive(Debug, Default, Clone, Copy)]
struct Box {
    r0: usize,
    r1: usize,
    g0: usize,
    g1: usize,
    b0: usize,
    b1: usize,
    vol: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Red,
    Green,
    Blue,
}

const INDEX_BITS: usize = 5;
const INDEX_COUNT: usize = (1 << INDEX_BITS) + 1; // 33
const TOTAL_SIZE: usize = INDEX_COUNT * INDEX_COUNT * INDEX_COUNT; // 35937
const MAX_COLORS_LIMIT: usize = 256;

#[inline]
fn get_index(r: usize, g: usize, b: usize) -> usize {
    (r << (INDEX_BITS * 2)) + (r << (INDEX_BITS + 1)) + (g << INDEX_BITS) + r + g + b
}

fn construct_histogram(
    pixels: &[Argb],
    weights: &mut [i64],
    m_r: &mut [i64],
    m_g: &mut [i64],
    m_b: &mut [i64],
    moments: &mut [f64],
) {
    let bits_to_remove = 8 - INDEX_BITS;
    for &pixel in pixels {
        let red = red_from_argb(pixel) as usize;
        let green = green_from_argb(pixel) as usize;
        let blue = blue_from_argb(pixel) as usize;

        let index_r = (red >> bits_to_remove) + 1;
        let index_g = (green >> bits_to_remove) + 1;
        let index_b = (blue >> bits_to_remove) + 1;
        let index = get_index(index_r, index_g, index_b);

        weights[index] += 1;
        m_r[index] += red as i64;
        m_g[index] += green as i64;
        m_b[index] += blue as i64;
        moments[index] += ((red * red) + (green * green) + (blue * blue)) as f64;
    }
}

fn compute_moments(
    weights: &mut [i64],
    m_r: &mut [i64],
    m_g: &mut [i64],
    m_b: &mut [i64],
    moments: &mut [f64],
) {
    for r in 1..INDEX_COUNT {
        let mut area = [0i64; INDEX_COUNT];
        let mut area_r = [0i64; INDEX_COUNT];
        let mut area_g = [0i64; INDEX_COUNT];
        let mut area_b = [0i64; INDEX_COUNT];
        let mut area_2 = [0.0f64; INDEX_COUNT];

        for g in 1..INDEX_COUNT {
            let mut line = 0i64;
            let mut line_r = 0i64;
            let mut line_g = 0i64;
            let mut line_b = 0i64;
            let mut line_2 = 0.0f64;

            for b in 1..INDEX_COUNT {
                let index = get_index(r, g, b);
                line += weights[index];
                line_r += m_r[index];
                line_g += m_g[index];
                line_b += m_b[index];
                line_2 += moments[index];

                area[b] += line;
                area_r[b] += line_r;
                area_g[b] += line_g;
                area_b[b] += line_b;
                area_2[b] += line_2;

                let previous_index = get_index(r - 1, g, b);
                weights[index] = weights[previous_index] + area[b];
                m_r[index] = m_r[previous_index] + area_r[b];
                m_g[index] = m_g[previous_index] + area_g[b];
                m_b[index] = m_b[previous_index] + area_b[b];
                moments[index] = moments[previous_index] + area_2[b];
            }
        }
    }
}

#[inline]
fn top(cube: &Box, direction: Direction, position: usize, moment: &[i64]) -> i64 {
    match direction {
        Direction::Red => {
            moment[get_index(position, cube.g1, cube.b1)]
                - moment[get_index(position, cube.g1, cube.b0)]
                - moment[get_index(position, cube.g0, cube.b1)]
                + moment[get_index(position, cube.g0, cube.b0)]
        }
        Direction::Green => {
            moment[get_index(cube.r1, position, cube.b1)]
                - moment[get_index(cube.r1, position, cube.b0)]
                - moment[get_index(cube.r0, position, cube.b1)]
                + moment[get_index(cube.r0, position, cube.b0)]
        }
        Direction::Blue => {
            moment[get_index(cube.r1, cube.g1, position)]
                - moment[get_index(cube.r1, cube.g0, position)]
                - moment[get_index(cube.r0, cube.g1, position)]
                + moment[get_index(cube.r0, cube.g0, position)]
        }
    }
}

#[inline]
fn bottom(cube: &Box, direction: Direction, moment: &[i64]) -> i64 {
    match direction {
        Direction::Red => {
            -moment[get_index(cube.r0, cube.g1, cube.b1)]
                + moment[get_index(cube.r0, cube.g1, cube.b0)]
                + moment[get_index(cube.r0, cube.g0, cube.b1)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        }
        Direction::Green => {
            -moment[get_index(cube.r1, cube.g0, cube.b1)]
                + moment[get_index(cube.r1, cube.g0, cube.b0)]
                + moment[get_index(cube.r0, cube.g0, cube.b1)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        }
        Direction::Blue => {
            -moment[get_index(cube.r1, cube.g1, cube.b0)]
                + moment[get_index(cube.r1, cube.g0, cube.b0)]
                + moment[get_index(cube.r0, cube.g1, cube.b0)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        }
    }
}

#[inline]
fn vol(cube: &Box, moment: &[i64]) -> i64 {
    moment[get_index(cube.r1, cube.g1, cube.b1)]
        - moment[get_index(cube.r1, cube.g1, cube.b0)]
        - moment[get_index(cube.r1, cube.g0, cube.b1)]
        + moment[get_index(cube.r1, cube.g0, cube.b0)]
        - moment[get_index(cube.r0, cube.g1, cube.b1)]
        + moment[get_index(cube.r0, cube.g1, cube.b0)]
        + moment[get_index(cube.r0, cube.g0, cube.b1)]
        - moment[get_index(cube.r0, cube.g0, cube.b0)]
}

fn variance(
    cube: &Box,
    weights: &[i64],
    m_r: &[i64],
    m_g: &[i64],
    m_b: &[i64],
    moments: &[f64],
) -> f64 {
    let dr = vol(cube, m_r) as f64;
    let dg = vol(cube, m_g) as f64;
    let db = vol(cube, m_b) as f64;
    let xx = moments[get_index(cube.r1, cube.g1, cube.b1)]
        - moments[get_index(cube.r1, cube.g1, cube.b0)]
        - moments[get_index(cube.r1, cube.g0, cube.b1)]
        + moments[get_index(cube.r1, cube.g0, cube.b0)]
        - moments[get_index(cube.r0, cube.g1, cube.b1)]
        + moments[get_index(cube.r0, cube.g1, cube.b0)]
        + moments[get_index(cube.r0, cube.g0, cube.b1)]
        - moments[get_index(cube.r0, cube.g0, cube.b0)];
    let hypotenuse = dr * dr + dg * dg + db * db;
    let volume = vol(cube, weights) as f64;
    xx - hypotenuse / volume
}

#[allow(clippy::too_many_arguments)]
fn maximize(
    cube: &Box,
    direction: Direction,
    first: usize,
    last: usize,
    cut: &mut i32,
    whole_w: i64,
    whole_r: i64,
    whole_g: i64,
    whole_b: i64,
    weights: &[i64],
    m_r: &[i64],
    m_g: &[i64],
    m_b: &[i64],
) -> f64 {
    let bottom_r = bottom(cube, direction, m_r);
    let bottom_g = bottom(cube, direction, m_g);
    let bottom_b = bottom(cube, direction, m_b);
    let bottom_w = bottom(cube, direction, weights);

    let mut max = 0.0;
    *cut = -1;

    for i in first..last {
        let mut half_r = bottom_r + top(cube, direction, i, m_r);
        let mut half_g = bottom_g + top(cube, direction, i, m_g);
        let mut half_b = bottom_b + top(cube, direction, i, m_b);
        let mut half_w = bottom_w + top(cube, direction, i, weights);
        if half_w == 0 {
            continue;
        }

        let mut temp =
            ((half_r * half_r + half_g * half_g + half_b * half_b) as f64) / (half_w as f64);

        half_r = whole_r - half_r;
        half_g = whole_g - half_g;
        half_b = whole_b - half_b;
        half_w = whole_w - half_w;
        if half_w == 0 {
            continue;
        }

        temp += ((half_r * half_r + half_g * half_g + half_b * half_b) as f64) / (half_w as f64);

        if temp > max {
            max = temp;
            *cut = i as i32;
        }
    }

    max
}

fn cut(
    box1: &mut Box,
    box2: &mut Box,
    weights: &[i64],
    m_r: &[i64],
    m_g: &[i64],
    m_b: &[i64],
) -> bool {
    let whole_r = vol(box1, m_r);
    let whole_g = vol(box1, m_g);
    let whole_b = vol(box1, m_b);
    let whole_w = vol(box1, weights);

    let mut cut_r = 0;
    let mut cut_g = 0;
    let mut cut_b = 0;

    let max_r = maximize(
        box1,
        Direction::Red,
        box1.r0 + 1,
        box1.r1,
        &mut cut_r,
        whole_w,
        whole_r,
        whole_g,
        whole_b,
        weights,
        m_r,
        m_g,
        m_b,
    );
    let max_g = maximize(
        box1,
        Direction::Green,
        box1.g0 + 1,
        box1.g1,
        &mut cut_g,
        whole_w,
        whole_r,
        whole_g,
        whole_b,
        weights,
        m_r,
        m_g,
        m_b,
    );
    let max_b = maximize(
        box1,
        Direction::Blue,
        box1.b0 + 1,
        box1.b1,
        &mut cut_b,
        whole_w,
        whole_r,
        whole_g,
        whole_b,
        weights,
        m_r,
        m_g,
        m_b,
    );

    let direction = if max_r >= max_g && max_r >= max_b {
        if cut_r < 0 {
            return false;
        }
        Direction::Red
    } else if max_g >= max_r && max_g >= max_b {
        Direction::Green
    } else {
        Direction::Blue
    };

    box2.r1 = box1.r1;
    box2.g1 = box1.g1;
    box2.b1 = box1.b1;

    match direction {
        Direction::Red => {
            box1.r1 = cut_r as usize;
            box2.r0 = cut_r as usize;
            box2.g0 = box1.g0;
            box2.b0 = box1.b0;
        }
        Direction::Green => {
            box2.r0 = box1.r0;
            box1.g1 = cut_g as usize;
            box2.g0 = cut_g as usize;
            box2.b0 = box1.b0;
        }
        Direction::Blue => {
            box2.r0 = box1.r0;
            box2.g0 = box1.g0;
            box1.b1 = cut_b as usize;
            box2.b0 = cut_b as usize;
        }
    }

    box1.vol = (box1.r1 - box1.r0) * (box1.g1 - box1.g0) * (box1.b1 - box1.b0);
    box2.vol = (box2.r1 - box2.r0) * (box2.g1 - box2.g0) * (box2.b1 - box2.b0);
    true
}

/// Quantizes an array of ARGB pixels using Wu's color quantizer into at most `max_colors`.
pub fn quantize_wu(pixels: &[Argb], mut max_colors: u16) -> Vec<Argb> {
    if max_colors == 0 || pixels.is_empty() {
        return Vec::new();
    }
    if max_colors > 256 {
        max_colors = 256;
    }
    let mut max_colors = max_colors as usize;

    let mut weights = vec![0i64; TOTAL_SIZE];
    let mut moments_red = vec![0i64; TOTAL_SIZE];
    let mut moments_green = vec![0i64; TOTAL_SIZE];
    let mut moments_blue = vec![0i64; TOTAL_SIZE];
    let mut moments = vec![0.0f64; TOTAL_SIZE];

    construct_histogram(
        pixels,
        &mut weights,
        &mut moments_red,
        &mut moments_green,
        &mut moments_blue,
        &mut moments,
    );
    compute_moments(
        &mut weights,
        &mut moments_red,
        &mut moments_green,
        &mut moments_blue,
        &mut moments,
    );

    let mut cubes = vec![Box::default(); MAX_COLORS_LIMIT];
    cubes[0].r0 = 0;
    cubes[0].g0 = 0;
    cubes[0].b0 = 0;
    cubes[0].r1 = INDEX_COUNT - 1;
    cubes[0].g1 = INDEX_COUNT - 1;
    cubes[0].b1 = INDEX_COUNT - 1;

    let mut volume_variance = vec![0.0f64; MAX_COLORS_LIMIT];
    let mut next = 0;
    let mut i = 1;
    while i < max_colors {
        let (left_slice, right_slice) = cubes.split_at_mut(i);
        let box1 = &mut left_slice[next];
        let box2 = &mut right_slice[0];

        if cut(
            box1,
            box2,
            &weights,
            &moments_red,
            &moments_green,
            &moments_blue,
        ) {
            volume_variance[next] = if cubes[next].vol > 1 {
                variance(
                    &cubes[next],
                    &weights,
                    &moments_red,
                    &moments_green,
                    &moments_blue,
                    &moments,
                )
            } else {
                0.0
            };
            volume_variance[i] = if cubes[i].vol > 1 {
                variance(
                    &cubes[i],
                    &weights,
                    &moments_red,
                    &moments_green,
                    &moments_blue,
                    &moments,
                )
            } else {
                0.0
            };
        } else {
            volume_variance[next] = 0.0;
            i = i.saturating_sub(1);
        }

        next = 0;
        let mut temp = volume_variance[0];
        for (j, &var) in volume_variance.iter().enumerate().take(i + 1).skip(1) {
            if var > temp {
                temp = var;
                next = j;
            }
        }
        if temp <= 0.0 {
            max_colors = i + 1;
            break;
        }
        i += 1;
    }

    let mut out_colors = Vec::new();
    for cube in cubes.iter().take(max_colors) {
        let weight = vol(cube, &weights);
        if weight > 0 {
            let red = (vol(cube, &moments_red) / weight) as i32;
            let green = (vol(cube, &moments_green) / weight) as i32;
            let blue = (vol(cube, &moments_blue) / weight) as i32;
            let argb = argb_from_rgb(red, green, blue);
            out_colors.push(argb);
        }
    }

    out_colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wu_two_red_three_green() {
        let pixels = vec![
            0xffff_0000,
            0xffff_0000,
            0xffff_0000,
            0xff00_ff00,
            0xff00_ff00,
        ];
        let result = quantize_wu(&pixels, 256);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_wu_single_colors() {
        let red_pixels = vec![0xffff_0000];
        let result = quantize_wu(&red_pixels, 256);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0xffff_0000);

        let green_pixels = vec![0xff00_ff00];
        let result = quantize_wu(&green_pixels, 256);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0xff00_ff00);

        let blue_pixels = vec![0xff00_00ff];
        let result = quantize_wu(&blue_pixels, 256);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0xff00_00ff);
    }

    #[test]
    fn test_wu_red_green_blue() {
        let pixels = vec![0xffff_0000, 0xff00_ff00, 0xff00_00ff];
        let result = quantize_wu(&pixels, 256);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 0xff00_00ff);
        assert_eq!(result[1], 0xffff_0000);
        assert_eq!(result[2], 0xff00_ff00);
    }
}
