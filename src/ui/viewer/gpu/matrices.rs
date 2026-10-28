use super::super::{
    Pose,
    math::{self, Mat4, Vec3},
};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Globals {
    model: Mat4,
    view_projection: Mat4,
    shadow: Mat4,
    camera: [f32; 4],
    viewport: [f32; 4],
    light: [f32; 4],
    post: [f32; 4],
}

impl Globals {
    pub fn new(pose: &Pose, side: u32, capacity: u32) -> Self {
        let distance = 1.1 / (0.42_f32 / 2.0).sin();
        let near = distance - 2.0;
        let far = distance + 2.0;
        let turn = math::mul(&math::rotation_x(pose.pitch), &math::rotation_y(pose.yaw));
        let focus = math::translation(pose.focus.map(|v| -v));
        let model = math::mul(
            &math::translation([0.0, pose.lift * 0.06, 0.0]),
            &math::mul(&turn, &focus),
        );
        let convert = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
        ];
        let projection = math::mul(
            &convert,
            &math::perspective(0.42 / pose.zoom.max(0.1), 1.0, near, far),
        );
        let view_projection = math::mul(&projection, &math::translation([0.0, 0.0, -distance]));
        let light = math::normalize([-0.5, 0.85, 0.65]);
        let eye = light.map(|v| v * 6.0);
        let light_view = look_at(eye, [0.0; 3]);
        let shadow = math::mul(&orthographic(2.25, 0.1, 12.0), &light_view);
        Self {
            model,
            view_projection,
            shadow,
            camera: [0.0, 0.0, distance, 0.0],
            viewport: [
                side as f32,
                side as f32,
                1.0 / 2048.0,
                side as f32 / capacity as f32,
            ],
            light: [light[0], light[1], light[2], 0.0],
            post: [near, far, 0.8, 0.0],
        }
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn look_at(eye: Vec3, center: Vec3) -> Mat4 {
    let forward = math::normalize(math::sub(eye, center));
    let right = math::normalize(cross([0.0, 1.0, 0.0], forward));
    let up = cross(forward, right);
    [
        right[0],
        up[0],
        forward[0],
        0.0,
        right[1],
        up[1],
        forward[1],
        0.0,
        right[2],
        up[2],
        forward[2],
        0.0,
        -math::dot(right, eye),
        -math::dot(up, eye),
        -math::dot(forward, eye),
        1.0,
    ]
}
fn orthographic(extent: f32, near: f32, far: f32) -> Mat4 {
    [
        1.0 / extent,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / extent,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / (near - far),
        0.0,
        0.0,
        0.0,
        near / (near - far),
        1.0,
    ]
}
