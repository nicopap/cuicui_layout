use bevy::math::Vec2Swizzles;
use bevy::prelude::{BVec2, Color, Gizmos, GlobalTransform, Vec2};
use bevy::utils::HashSet;

use super::{CameraQuery, RuleArrow};
use crate::debug::CHEVRON_RATIO;
use crate::direction::Axis;
use crate::{LayoutRect, Size};

trait ApproxF32 {
    fn is(self, other: f32) -> bool;
}
impl ApproxF32 for f32 {
    fn is(self, other: f32) -> bool {
        let diff = (self - other).abs();
        diff < 0.001
    }
}

fn rect_border_axis(rect: LayoutRect, margin: Size<f32>) -> (f32, f32, f32, f32) {
    let pos = rect.pos() + Vec2::from(margin);
    let size = Vec2::from(rect.size()) - Vec2::from(margin) * 2.;
    let offset = pos + size;
    (pos.x, offset.x, pos.y, offset.y)
}

/// Collection of axis aligned "lines" (actually just their coordinate on
/// a given axis).
#[derive(Debug, Clone)]
struct DrawnLines {
    lines: HashSet<i64>,
    width: f32,
}
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
impl DrawnLines {
    fn new(width: f32) -> Self {
        DrawnLines { lines: HashSet::new(), width }
    }
    /// Return `value` offset by as many `increment`s as necessary to make it
    /// not overlap with already drawn lines.
    fn inset(&self, value: f32, increment: i64) -> f32 {
        let scaled = value / self.width;
        let fract = scaled.fract();
        let mut on_grid = scaled.floor() as i64;
        loop {
            if !self.lines.contains(&on_grid) {
                return ((on_grid as f32) + fract) * self.width;
            }
            on_grid += increment;
        }
    }
    /// Remove a line from the collection of drawn lines.
    ///
    /// Typically, we only care for pre-existing lines when drawing the children
    /// of a container, nothing more. So we remove it after we are done with
    /// the children.
    fn remove(&mut self, value: f32, increment: i64) {
        let mut on_grid = (value / self.width).floor() as i64;
        loop {
            let next_cell = on_grid + increment;
            if !self.lines.contains(&next_cell) {
                self.lines.remove(&on_grid);
                return;
            }
            on_grid = next_cell;
        }
    }
    /// Add a line from the collection of drawn lines.
    fn add(&mut self, value: f32, increment: i64) {
        let mut on_grid = (value / self.width).floor() as i64;
        loop {
            let did_not_exist = self.lines.insert(on_grid);
            if did_not_exist {
                return;
            }
            on_grid += increment;
        }
    }
}

pub(super) struct InsetGizmo<'w, 's> {
    draw: Gizmos<'s>,
    cam: CameraQuery<'w, 's>,
    known_y: DrawnLines,
    known_x: DrawnLines,
}
impl<'w, 's> InsetGizmo<'w, 's> {
    pub(super) fn new(draw: Gizmos<'s>, cam: CameraQuery<'w, 's>, line_width: f32) -> Self {
        InsetGizmo {
            draw,
            cam,
            known_y: DrawnLines::new(line_width),
            known_x: DrawnLines::new(line_width),
        }
    }
    fn relative(&self, mut position: Vec2) -> Vec2 {
        let zero = GlobalTransform::IDENTITY;
        let Ok((cam, debug)) = self.cam.get_single() else { return Vec2::ZERO;};
        if debug.screen_space {
            if let Some(new_position) = cam.world_to_viewport(&zero, position.extend(0.)) {
                position = new_position;
            };
        }
        position.xy()
    }
    /// Draw rule at edge of container on given axis.
    pub(super) fn rule(
        &mut self,
        center: Vec2,
        extents: Vec2,
        rule: RuleArrow,
        axis: Axis,
        color: Color,
    ) {
        use crate::Flow::{Horizontal as Width, Vertical as Height};

        let select = BVec2::new(axis == Width, axis == Height);
        let c = center;
        let e = Vec2::select(select, extents, Vec2::ZERO);
        let trim_e = (e * 0.25).min(Vec2::splat(100.));

        let Some((start1, end1, _)) = rule.arrange(c - e + trim_e, c - e) else { return; };
        let Some((start2, end2, _)) = rule.arrange(c + e - trim_e, c + e) else { return; };
        self.arrow(start1, end1, color, start1.distance(end1) * CHEVRON_RATIO);
        self.arrow(start2, end2, color, start2.distance(end2) * CHEVRON_RATIO);
    }
    fn line_2d(&mut self, start: Vec2, end: Vec2, color: Color) {
        let (start, end) = (self.relative(start), self.relative(end));
        self.draw.line_2d(start, end, color);
    }
    pub(super) fn set_scope(&mut self, rect: LayoutRect, margin: Size<f32>) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        self.known_x.add(left, 1);
        self.known_x.add(right, -1);
        self.known_y.add(top, 1);
        self.known_y.add(bottom, -1);
    }
    pub(super) fn clear_scope(&mut self, rect: LayoutRect, margin: Size<f32>) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        self.known_x.remove(left, 1);
        self.known_x.remove(right, -1);
        self.known_y.remove(top, 1);
        self.known_y.remove(bottom, -1);
    }
    pub(super) fn rect_2d(&mut self, rect: LayoutRect, margin: Size<f32>, color: Color) {
        let (left, right, top, bottom) = rect_border_axis(rect, margin);
        if left.is(right) {
            self.line_2d(Vec2::new(left, top), Vec2::new(left, bottom), color);
        }
        if top.is(bottom) {
            self.line_2d(Vec2::new(left, top), Vec2::new(right, top), color);
        }
        let inset_x = |v, incr| self.known_x.inset(v, incr);
        let inset_y = |v, incr| self.known_y.inset(v, incr);
        let (left, right) = (inset_x(left, 1), inset_x(right, -1));
        let (top, bottom) = (inset_y(top, 1), inset_y(bottom, -1));
        self.draw.linestrip_2d(
            [
                Vec2::new(left, top),
                Vec2::new(left, bottom),
                Vec2::new(right, bottom),
                Vec2::new(right, top),
                Vec2::new(left, top),
            ]
            .map(|v| self.relative(v)),
            color,
        );
    }
    fn arrow(&mut self, start: Vec2, end: Vec2, color: Color, chevron_size: f32) {
        let Some(angle) = (end - start).try_normalize() else { return; };

        let top = Vec2::new(-1., 1.);
        let bottom = Vec2::new(-1., -1.);
        let len = chevron_size;
        self.line_2d(start, end, color);
        self.line_2d(end, end + angle.rotate(top) * len, color);
        self.line_2d(end, end + angle.rotate(bottom) * len, color);
    }
}