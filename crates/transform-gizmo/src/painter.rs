use std::f64::consts::TAU;

use egui::layers::ShapeIdx;
use egui::{Color32, Pos2, Rect, Shape, Stroke};
use glam::{DMat4, DVec3};

use crate::math::world_to_screen;

const STEPS_PER_RAD: f64 = 20.0;

pub struct Painter3d {
    painter: egui::Painter,
    mvp: DMat4,
    viewport: Rect,
}

impl Painter3d {
    pub fn new(painter: egui::Painter, mvp: DMat4, viewport: Rect) -> Self {
        Self {
            painter: painter.with_clip_rect(viewport),
            mvp,
            viewport,
        }
    }

    fn arc_points(&self, radius: f64, start_angle: f64, end_angle: f64) -> Vec<Pos2> {
        let angle = f64::clamp(end_angle - start_angle, -TAU, TAU);

        let step_count = steps(angle);
        let mut points = Vec::with_capacity(step_count);

        let step_size = angle / (step_count - 1) as f64;

        for step in (0..step_count).map(|i| step_size * i as f64) {
            let x = f64::cos(start_angle + step) * radius;
            let z = f64::sin(start_angle + step) * radius;

            points.push(DVec3::new(x, 0.0, z));
        }

        points
            .into_iter()
            .filter_map(|point| self.vec3_to_pos2(point))
            .collect::<Vec<_>>()
    }

    pub fn arc(
        &self,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        let mut points = self.arc_points(radius, start_angle, end_angle);

        let closed = points
            .first()
            .zip(points.last())
            .filter(|(first, last)| first.distance(**last) < 1e-2)
            .is_some();

        if closed {
            points.pop();
            self.painter.add(Shape::closed_line(points, stroke))
        } else {
            self.painter.add(Shape::line(points, stroke))
        }
    }

    pub fn circle(&self, radius: f64, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.arc(radius, 0.0, TAU, stroke)
    }

    pub fn filled_circle(&self, radius: f64, color: Color32) -> ShapeIdx {
        let mut points = self.arc_points(radius, 0.0, TAU);
        points.pop();

        self.painter
            .add(Shape::convex_polygon(points, color, Stroke::NONE))
    }

    pub fn line_segment(&self, from: DVec3, to: DVec3, stroke: impl Into<Stroke>) {
        let mut points: [Pos2; 2] = Default::default();

        for (i, point) in points.iter_mut().enumerate() {
            if let Some(pos) = world_to_screen(self.viewport, self.mvp, [from, to][i]) {
                *point = pos;
            } else {
                return;
            }
        }

        self.painter.line_segment(points, stroke);
    }

    pub fn arrow(&self, from: DVec3, to: DVec3, stroke: impl Into<Stroke>) {
        let stroke = stroke.into();
        let arrow_start = world_to_screen(self.viewport, self.mvp, from);
        let arrow_end = world_to_screen(self.viewport, self.mvp, to);

        if let Some((start, end)) = arrow_start.zip(arrow_end) {
            let cross = (end - start).normalized().rot90() * stroke.width;

            self.painter.add(Shape::convex_polygon(
                vec![start - cross, start + cross, end],
                stroke.color,
                Stroke::NONE,
            ));
        }
    }

    pub fn polygon(&self, points: &[DVec3], fill: impl Into<Color32>, stroke: impl Into<Stroke>) {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        if points.len() > 2 {
            self.painter
                .add(Shape::convex_polygon(points, fill, stroke));
        }
    }

    pub fn polyline(&self, points: &[DVec3], stroke: impl Into<Stroke>) {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        if points.len() > 1 {
            self.painter.add(Shape::line(points, stroke));
        }
    }

    fn vec3_to_pos2(&self, vec: DVec3) -> Option<Pos2> {
        world_to_screen(self.viewport, self.mvp, vec)
    }
}

fn steps(angle: f64) -> usize {
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}
