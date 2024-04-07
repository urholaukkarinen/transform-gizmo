pub use emath::{Pos2, Rect, Vec2};
pub use glam::{DMat3, DMat4, DQuat, DVec2, DVec3, DVec4, Mat4, Quat, Vec3, Vec4Swizzles};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Transform {
    pub scale: mint::Vector3<f64>,
    pub rotation: mint::Quaternion<f64>,
    pub translation: mint::Vector3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: DVec3::ONE.into(),
            rotation: DQuat::IDENTITY.into(),
            translation: DVec3::ZERO.into(),
        }
    }
}

impl Transform {
    pub fn from_scale_rotation_translation(
        scale: impl Into<mint::Vector3<f64>>,
        rotation: impl Into<mint::Quaternion<f64>>,
        translation: impl Into<mint::Vector3<f64>>,
    ) -> Self {
        Self {
            scale: scale.into(),
            rotation: rotation.into(),
            translation: translation.into(),
        }
    }
}

/// Creates a matrix that represents rotation between two 3d vectors
///
/// Credit: <https://www.iquilezles.org/www/articles/noacos/noacos.htm>
pub(crate) fn rotation_align(from: DVec3, to: DVec3) -> DMat3 {
    let v = from.cross(to);
    let c = from.dot(to);
    let k = 1.0 / (1.0 + c);

    DMat3::from_cols_array(&[
        v.x * v.x * k + c,
        v.x * v.y * k + v.z,
        v.x * v.z * k - v.y,
        v.y * v.x * k - v.z,
        v.y * v.y * k + c,
        v.y * v.z * k + v.x,
        v.z * v.x * k + v.y,
        v.z * v.y * k - v.x,
        v.z * v.z * k + c,
    ])
}

/// Finds points on two rays that are closest to each other.
/// This can be used to determine the shortest distance between those two rays.
///
/// Credit: Practical Geometry Algorithms by Daniel Sunday: <http://geomalgorithms.com/code.html>
pub(crate) fn ray_to_ray(a1: DVec3, adir: DVec3, b1: DVec3, bdir: DVec3) -> (f64, f64) {
    let b = adir.dot(bdir);
    let w = a1 - b1;
    let d = adir.dot(w);
    let e = bdir.dot(w);
    let dot = 1.0 - b * b;
    let ta;
    let tb;

    if dot < 1e-8 {
        ta = 0.0;
        tb = e;
    } else {
        ta = (b * e - d) / dot;
        tb = (e - b * d) / dot;
    }

    (ta, tb)
}

/// Finds points on two segments that are closest to each other.
/// This can be used to determine the shortest distance between those two segments.
///
/// Credit: Practical Geometry Algorithms by Daniel Sunday: <http://geomalgorithms.com/code.html>
pub(crate) fn segment_to_segment(a1: DVec3, a2: DVec3, b1: DVec3, b2: DVec3) -> (f64, f64) {
    let da = a2 - a1;
    let db = b2 - b1;
    let la = da.length_squared();
    let lb = db.length_squared();
    let dd = da.dot(db);
    let d1 = a1 - b1;
    let d = da.dot(d1);
    let e = db.dot(d1);
    let n = la * lb - dd * dd;

    let mut sn;
    let mut tn;
    let mut sd = n;
    let mut td = n;

    if n < 1e-8 {
        sn = 0.0;
        sd = 1.0;
        tn = e;
        td = lb;
    } else {
        sn = dd * e - lb * d;
        tn = la * e - dd * d;
        if sn < 0.0 {
            sn = 0.0;
            tn = e;
            td = lb;
        } else if sn > sd {
            sn = sd;
            tn = e + dd;
            td = lb;
        }
    }

    if tn < 0.0 {
        tn = 0.0;
        if -d < 0.0 {
            sn = 0.0;
        } else if -d > la {
            sn = sd;
        } else {
            sn = -d;
            sd = la;
        }
    } else if tn > td {
        tn = td;
        if (-d + dd) < 0.0 {
            sn = 0.0;
        } else if (-d + dd) > la {
            sn = sd;
        } else {
            sn = -d + dd;
            sd = la;
        }
    }

    let ta = if sn.abs() < 1e-8 { 0.0 } else { sn / sd };
    let tb = if tn.abs() < 1e-8 { 0.0 } else { tn / td };

    (ta, tb)
}

/// Finds the intersection point of a ray and a plane
pub(crate) fn intersect_plane(
    plane_normal: DVec3,
    plane_origin: DVec3,
    ray_origin: DVec3,
    ray_dir: DVec3,
    t: &mut f64,
) -> bool {
    let denom = plane_normal.dot(ray_dir);

    if denom.abs() < 10e-8 {
        false
    } else {
        *t = (plane_origin - ray_origin).dot(plane_normal) / denom;
        *t >= 0.0
    }
}

/// Finds the intersection point of a ray and a plane
/// and distance from the intersection to the plane origin
pub(crate) fn ray_to_plane_origin(
    disc_normal: DVec3,
    disc_origin: DVec3,
    ray_origin: DVec3,
    ray_dir: DVec3,
) -> (f64, f64) {
    let mut t = 0.0;
    if intersect_plane(disc_normal, disc_origin, ray_origin, ray_dir, &mut t) {
        let p = ray_origin + ray_dir * t;
        let v = p - disc_origin;
        let d2 = v.dot(v);
        (t, f64::sqrt(d2))
    } else {
        (t, f64::MAX)
    }
}

/// Rounds given value to the nearest interval
pub(crate) fn round_to_interval(val: f64, interval: f64) -> f64 {
    (val / interval).round() * interval
}

/// Calculates 2d screen coordinates from 3d world coordinates
pub(crate) fn world_to_screen(viewport: Rect, mvp: DMat4, pos: DVec3) -> Option<Pos2> {
    let mut pos = mvp * DVec4::from((pos, 1.0));

    if pos.w < 1e-10 {
        return None;
    }

    pos /= pos.w;
    pos.y *= -1.0;

    let center = viewport.center();

    Some(Pos2::new(
        (center.x as f64 + pos.x * viewport.width() as f64 / 2.0) as f32,
        (center.y as f64 + pos.y * viewport.height() as f64 / 2.0) as f32,
    ))
}

/// Calculates 3d world coordinates from 2d screen coordinates
pub(crate) fn screen_to_world(viewport: Rect, mat: DMat4, pos: Pos2, z: f64) -> DVec3 {
    let x = (((pos.x - viewport.min.x) / viewport.width()) * 2.0 - 1.0) as f64;
    let y = (((pos.y - viewport.min.y) / viewport.height()) * 2.0 - 1.0) as f64;

    let mut world_pos = mat * DVec4::new(x, -y, z, 1.0);

    // w is zero when far plane is set to infinity
    if world_pos.w.abs() < 1e-7 {
        world_pos.w = 1e-7;
    }

    world_pos /= world_pos.w;

    world_pos.xyz()
}
