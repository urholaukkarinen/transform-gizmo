use std::f32::consts::TAU;

use egui::layers::ShapeIdx;
use egui::{Color32, Pos2, Rect, Shape, Stroke};
use glam::{Mat4, Vec3};

use crate::math::world_to_screen;

const STEPS_PER_RAD: f32 = 20.0;

pub struct Painter3d {
    painter: egui::Painter,
    mvp: Mat4,
    viewport: Rect,
}

impl Painter3d {
    pub fn new(painter: egui::Painter, mvp: Mat4, viewport: Rect) -> Self {
        Self {
            painter,
            mvp,
            viewport,
        }
    }

    pub fn arc(
        &self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        let angle = end_angle - start_angle;
        let step_count = steps(angle);
        let mut points = Vec::with_capacity(step_count);

        let step_size = angle / (step_count - 1) as f32;

        for step in (0..step_count).map(|i| step_size * i as f32) {
            let x = f32::cos(start_angle + step) * radius;
            let z = f32::sin(start_angle + step) * radius;

            points.push(Vec3::new(x, 0.0, z));
        }

        let points = points
            .into_iter()
            .filter_map(|point| self.vec3_to_pos2(point))
            .collect::<Vec<_>>();

        self.painter.add(Shape::line(points, stroke))
    }

    pub fn circle(&self, radius: f32, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.arc(radius, 0.0, TAU, stroke)
    }

    pub fn line_segment(&self, from: Vec3, to: Vec3, stroke: impl Into<Stroke>) {
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

    pub fn arrow(&self, from: Vec3, to: Vec3, stroke: impl Into<Stroke>) {
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

    pub fn polygon(&self, points: &[Vec3], fill: impl Into<Color32>, stroke: impl Into<Stroke>) {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        if points.len() > 2 {
            self.painter
                .add(Shape::convex_polygon(points, fill, stroke));
        }
    }

    pub fn polyline(&self, points: &[Vec3], stroke: impl Into<Stroke>) {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        if points.len() > 1 {
            self.painter.add(Shape::line(points, stroke));
        }
    }

    fn vec3_to_pos2(&self, vec: Vec3) -> Option<Pos2> {
        world_to_screen(self.viewport, self.mvp, vec)
    }
}

fn steps(angle: f32) -> usize {
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}
