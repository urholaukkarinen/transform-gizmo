use std::f64::consts::TAU;

use crate::math::{Pos2, Rect};
use ecolor::Color32;
use epaint::{Mesh, TessellationOptions, Tessellator, TextureId};
pub(crate) use epaint::{PathStroke, Shape, Stroke};
use glam::{DMat4, DVec3};

use crate::math::world_to_screen;

const STEPS_PER_RAD: f64 = 20.0;

pub(crate) struct ShapeBuidler {
    mvp: DMat4,
    viewport: Rect,
    pixels_per_point: f32,
}

impl ShapeBuidler {
    pub(crate) fn new(mvp: DMat4, viewport: Rect, pixels_per_point: f32) -> Self {
        Self {
            mvp,
            viewport,
            pixels_per_point,
        }
    }

    fn tessellate_shape(&self, shape: Shape) -> Mesh {
        let mut tessellator = Tessellator::new(
            self.pixels_per_point,
            TessellationOptions {
                feathering: true,
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        );

        let mut mesh = Mesh::default();
        tessellator.tessellate_shape(shape, &mut mesh);

        mesh.texture_id = TextureId::default();
        mesh
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

    pub(crate) fn arc(
        &self,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        stroke: impl Into<PathStroke>,
    ) -> Mesh {
        let mut points = self.arc_points(radius, start_angle, end_angle);

        let closed = points
            .first()
            .zip(points.last())
            .filter(|(first, last)| first.distance(**last) < 1e-2)
            .is_some();

        self.tessellate_shape(if closed {
            points.pop();
            Shape::closed_line(points, stroke.into())
        } else {
            Shape::line(points, stroke.into())
        })
    }

    pub(crate) fn circle(&self, radius: f64, stroke: impl Into<PathStroke>) -> Mesh {
        self.arc(radius, 0.0, TAU, stroke)
    }

    pub(crate) fn filled_circle(
        &self,
        radius: f64,
        color: Color32,
        stroke: impl Into<PathStroke>,
    ) -> Mesh {
        let mut points = self.arc_points(radius, 0.0, TAU);
        points.pop();

        self.tessellate_shape(Shape::convex_polygon(points, color, stroke.into()))
    }

    pub(crate) fn line_segment(
        &self,
        from: DVec3,
        to: DVec3,
        stroke: impl Into<PathStroke>,
    ) -> Mesh {
        let mut points: [Pos2; 2] = Default::default();

        for (i, point) in points.iter_mut().enumerate() {
            if let Some(pos) = world_to_screen(self.viewport, self.mvp, [from, to][i]) {
                *point = pos;
            } else {
                return Mesh::default();
            }
        }

        self.tessellate_shape(Shape::LineSegment {
            points,
            stroke: stroke.into(),
        })
    }

    pub(crate) fn arrow(&self, from: DVec3, to: DVec3, stroke: impl Into<Stroke>) -> Mesh {
        let stroke = stroke.into();
        let arrow_start = world_to_screen(self.viewport, self.mvp, from);
        let arrow_end = world_to_screen(self.viewport, self.mvp, to);

        self.tessellate_shape(if let Some((start, end)) = arrow_start.zip(arrow_end) {
            let cross = (end - start).normalized().rot90() * stroke.width / 2.0;

            Shape::convex_polygon(
                vec![start - cross, start + cross, end],
                stroke.color,
                PathStroke::NONE,
            )
        } else {
            Shape::Noop
        })
    }

    pub(crate) fn polygon(
        &self,
        points: &[DVec3],
        fill: impl Into<Color32>,
        stroke: impl Into<PathStroke>,
    ) -> Mesh {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        self.tessellate_shape(if points.len() > 2 {
            Shape::convex_polygon(points, fill, stroke)
        } else {
            Shape::Noop
        })
    }

    pub(crate) fn polyline(&self, points: &[DVec3], stroke: impl Into<PathStroke>) -> Mesh {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.viewport, self.mvp, *pos))
            .collect::<Vec<_>>();

        self.tessellate_shape(if points.len() > 1 {
            Shape::line(points, stroke)
        } else {
            Shape::Noop
        })
    }

    pub(crate) fn sector(
        &self,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        fill: impl Into<Color32>,
        stroke: impl Into<PathStroke>,
    ) -> Mesh {
        let angle_delta = end_angle - start_angle;
        let step_count = steps(angle_delta.abs());

        if step_count < 2 {
            return Mesh::default();
        }

        let mut points = Vec::with_capacity(step_count + 1);

        let step_size = angle_delta / (step_count - 1) as f64;

        if ((start_angle - end_angle).abs() - TAU).abs() < step_size.abs() {
            return self.filled_circle(radius, fill.into(), stroke);
        }

        points.push(DVec3::new(0.0, 0.0, 0.0));

        let (sin_step, cos_step) = step_size.sin_cos();
        let (mut sin_angle, mut cos_angle) = start_angle.sin_cos();

        for _ in 0..step_count {
            let x = cos_angle * radius;
            let z = sin_angle * radius;

            points.push(DVec3::new(x, 0.0, z));

            let new_sin = sin_angle * cos_step + cos_angle * sin_step;
            let new_cos = cos_angle * cos_step - sin_angle * sin_step;

            sin_angle = new_sin;
            cos_angle = new_cos;
        }

        let points = points
            .into_iter()
            .filter_map(|point| self.vec3_to_pos2(point))
            .collect::<Vec<_>>();

        self.tessellate_shape(Shape::convex_polygon(points, fill, stroke))
    }

    fn vec3_to_pos2(&self, vec: DVec3) -> Option<Pos2> {
        world_to_screen(self.viewport, self.mvp, vec)
    }
}

fn steps(angle: f64) -> usize {
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}
