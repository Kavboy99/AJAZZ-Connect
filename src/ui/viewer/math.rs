//! The few vector and matrix operations the model view needs. Matrices are column-major, as
//! glTF stores them and OpenGL reads them.

pub type Vec3 = [f32; 3];
pub type Mat4 = [f32; 16];

pub const IDENTITY: Mat4 = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn normalize(a: Vec3) -> Vec3 {
    let length = dot(a, a).sqrt();
    if length > f32::EPSILON {
        [a[0] / length, a[1] / length, a[2] / length]
    } else {
        a
    }
}

pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            out[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    out
}

pub fn transform_point(m: &Mat4, p: Vec3) -> Vec3 {
    let w = m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15];
    let w = if w.abs() > f32::EPSILON { w } else { 1.0 };
    [
        (m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12]) / w,
        (m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13]) / w,
        (m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14]) / w,
    ]
}

/// Directions ignore translation; good enough for the rigid transforms of a model file.
pub fn transform_direction(m: &Mat4, d: Vec3) -> Vec3 {
    normalize([
        m[0] * d[0] + m[4] * d[1] + m[8] * d[2],
        m[1] * d[0] + m[5] * d[1] + m[9] * d[2],
        m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
    ])
}

/// glTF node transform: translation, rotation quaternion `[x, y, z, w]`, scale.
pub fn from_trs(t: Vec3, q: [f32; 4], s: Vec3) -> Mat4 {
    let [x, y, z, w] = q;
    [
        (1.0 - 2.0 * (y * y + z * z)) * s[0],
        (2.0 * (x * y + z * w)) * s[0],
        (2.0 * (x * z - y * w)) * s[0],
        0.0,
        (2.0 * (x * y - z * w)) * s[1],
        (1.0 - 2.0 * (x * x + z * z)) * s[1],
        (2.0 * (y * z + x * w)) * s[1],
        0.0,
        (2.0 * (x * z + y * w)) * s[2],
        (2.0 * (y * z - x * w)) * s[2],
        (1.0 - 2.0 * (x * x + y * y)) * s[2],
        0.0,
        t[0],
        t[1],
        t[2],
        1.0,
    ]
}

pub fn rotation_x(angle: f32) -> Mat4 {
    let (s, c) = angle.sin_cos();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn rotation_y(angle: f32) -> Mat4 {
    let (s, c) = angle.sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn translation(t: Vec3) -> Mat4 {
    let mut m = IDENTITY;
    m[12..15].copy_from_slice(&t);
    m
}

pub fn scale(s: f32) -> Mat4 {
    let mut m = IDENTITY;
    m[0] = s;
    m[5] = s;
    m[10] = s;
    m
}

/// Right-handed perspective, depth mapped to -1..1.
pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y / 2.0).tan();
    let mut m = [0.0; 16];
    m[0] = f / aspect;
    m[5] = f;
    m[10] = (far + near) / (near - far);
    m[11] = -1.0;
    m[14] = 2.0 * far * near / (near - far);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_compose_and_project() {
        let quarter = rotation_y(std::f32::consts::FRAC_PI_2);
        let p = transform_point(&quarter, [1.0, 0.0, 0.0]);
        assert!((p[2] + 1.0).abs() < 1e-5 && p[0].abs() < 1e-5);
        let q = from_trs([1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0], [2.0; 3]);
        assert_eq!(transform_point(&q, [1.0, 1.0, 1.0]), [3.0, 4.0, 5.0]);
        assert_eq!(mul(&IDENTITY, &q), q);
        let projected = transform_point(&perspective(1.0, 1.0, 1.0, 10.0), [0.0, 0.0, -1.0]);
        assert!((projected[2] + 1.0).abs() < 1e-5);
    }
}
