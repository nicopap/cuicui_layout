use bevy::math::Vec2Swizzles;
use bevy::prelude::{BVec2, Color, Gizmos, GlobalTransform, Vec2};
use bevy::utils::HashMap;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Dir {
    Start,
    End,
}
impl Dir {
    const fn increments(self) -> i64 {
        match self {
            Dir::Start => 1,
            Dir::End => -1,
        }
    }
}
impl From<i64> for Dir {
    fn from(value: i64) -> Self {
        if value.is_positive() {
            Dir::Start
        } else {
            Dir::End
        }
    }
}
/// Collection of axis aligned "lines" (actually just their coordinate on
/// a given axis).
#[derive(Debug, Clone)]
struct DrawnLines {
    lines: HashMap<i64, Dir>,
    width: f32,
}
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
impl DrawnLines {
    fn new(width: f32) -> Self {
        DrawnLines { lines: HashMap::new(), width }
    }
    /// Return `value` offset by as many `increment`s as necessary to make it
    /// not overlap with already drawn lines.
    fn inset(&self, value: f32) -> f32 {
        let scaled = value / self.width;
        let fract = scaled.fract();
        let mut on_grid = scaled.floor() as i64;
        for _ in 0..10 {
            let Some(dir) = self.lines.get(&on_grid) else {
                break;
            };
            // TODO(clean): This fixes a panic, but I'm not sure how valid this is
            let Some(added) = on_grid.checked_add(dir.increments()) else {
                break;
            };
            on_grid = added;
        }
        ((on_grid as f32) + fract) * self.width
    }
    /// Remove a line from the collection of drawn lines.
    ///
    /// Typically, we only care for pre-existing lines when drawing the children
    /// of a container, nothing more. So we remove it after we are done with
    /// the children.
    fn remove(&mut self, value: f32, increment: i64) {
        let mut on_grid = (value / self.width).floor() as i64;
        loop {
            // TODO(clean): This fixes a panic, but I'm not sure how valid this is
            let Some(next_cell) = on_grid.checked_add(increment) else {
                return;
            };
            if !self.lines.contains_key(&next_cell) {
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
            let old_value = self.lines.insert(on_grid, increment.into());
            if old_value.is_none() {
                return;
            }
            // TODO(clean): This fixes a panic, but I'm not sure how valid this is
            let Some(added) = on_grid.checked_add(increment) else {
                return;
            };
            on_grid = added;
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
        let Ok((cam, debug)) = self.cam.get_single() else {
            return Vec2::ZERO;
        };
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

        let Some((start1, end1, _)) = rule.arrange(c - e + trim_e, c - e) else {
            return;
        };
        let Some((start2, end2, _)) = rule.arrange(c + e - trim_e, c + e) else {
            return;
        };
        self.arrow(start1, end1, color, start1.distance(end1) * CHEVRON_RATIO);
        self.arrow(start2, end2, color, start2.distance(end2) * CHEVRON_RATIO);
    }
    fn line_2d(&mut self, mut start: Vec2, mut end: Vec2, color: Color) {
        if start.x.is(end.x) {
            start.x = self.known_x.inset(start.x);
            end.x = start.x;
        } else if start.y.is(end.y) {
            start.y = self.known_y.inset(start.y);
            end.y = start.y;
        }
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
        } else if top.is(bottom) {
            self.line_2d(Vec2::new(left, top), Vec2::new(right, top), color);
        } else {
            let inset_x = |v| self.known_x.inset(v);
            let inset_y = |v| self.known_y.inset(v);
            let (left, right) = (inset_x(left), inset_x(right));
            let (top, bottom) = (inset_y(top), inset_y(bottom));
            let strip = [
                Vec2::new(left, top),
                Vec2::new(left, bottom),
                Vec2::new(right, bottom),
                Vec2::new(right, top),
                Vec2::new(left, top),
            ];
            self.draw
                .linestrip_2d(strip.map(|v| self.relative(v)), color);
        }
    }
    fn arrow(&mut self, start: Vec2, end: Vec2, color: Color, chevron_size: f32) {
        let Some(angle) = (end - start).try_normalize() else {
            return;
        };
        let top = Vec2::new(-1., 1.);
        let bottom = Vec2::new(-1., -1.);
        let len = chevron_size;
        self.line_2d(start, end, color);
        self.line_2d(end, end + angle.rotate(top) * len, color);
        self.line_2d(end, end + angle.rotate(bottom) * len, color);
    }
}
