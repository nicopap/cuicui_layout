#![allow(
    clippy::float_cmp,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
// ignore: this is a quick-off. The suggested fixes are soooo noisy.

use bevy::prelude::Color;
use hsluv::{hsluv_to_rgb, rgb_to_hsluv};

fn lerp32(a: f32, b: f32, lerp: f32) -> f32 {
    let spread = b - a;
    spread.mul_add(lerp, a)
}
fn lerp64(a: f64, b: f64, lerp: f64) -> f64 {
    let spread = b - a;
    spread.mul_add(lerp, a)
}
pub fn color_lerp(from: Color, to: Color, lerp: f64) -> Color {
    let [r1, g1, b1, a1] = from.as_rgba_f32();
    let [r2, g2, b2, a2] = to.as_rgba_f32();
    let a_out = lerp32(a1, a2, lerp as f32);
    if r1 == r2 && g1 == g2 && b1 == b2 {
        Color::rgba(r1, g1, b1, a_out)
    } else {
        let d = lerp;
        let (h1, s1, l1) = rgb_to_hsluv((r1 as f64, g1 as f64, b1 as f64));
        let (h2, s2, l2) = rgb_to_hsluv((r2 as f64, g2 as f64, b2 as f64));
        let hsluv = (lerp64(h1, h2, d), lerp64(s1, s2, d), lerp64(l1, l2, d));
        let (r_out, g_out, b_out) = hsluv_to_rgb(hsluv);
        Color::rgba(r_out as f32, g_out as f32, b_out as f32, a_out)
    }
}
