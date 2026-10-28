use super::super::math::{self, Mat4, Vec3};
use super::{Pose, Scene};
const FIELD_OF_VIEW: f32 = 0.42;
/// Distance at which a unit sphere just fits, times a margin for the lift and the tilt.
const FIT_MARGIN: f32 = 1.1;
/// How high a fully lifted model floats, in model radii.
const LIFT: f32 = 0.06;
pub(super) struct Camera {
    pub model: Mat4,
    /// Model to clip space in one matrix.
    mvp: Mat4,
    half: f32,
}

impl Camera {
    pub fn new(scene: &Scene, pose: &Pose, side: usize) -> Self {
        let distance = FIT_MARGIN / (FIELD_OF_VIEW / 2.0).sin();
        let normalize = math::mul(
            &math::scale(1.0 / scene.radius),
            &math::translation([-scene.center[0], -scene.center[1], -scene.center[2]]),
        );
        let turn = math::mul(&math::rotation_x(pose.pitch), &math::rotation_y(pose.yaw));
        let rise = math::translation([0.0, pose.lift * LIFT, 0.0]);
        let focus = math::translation(pose.focus.map(|v| -v));
        let model = math::mul(&rise, &math::mul(&turn, &math::mul(&focus, &normalize)));
        let clip = math::mul(
            &math::perspective(
                FIELD_OF_VIEW / pose.zoom.max(0.1),
                1.0,
                distance - 2.0,
                distance + 2.0,
            ),
            &math::translation([0.0, 0.0, -distance]),
        );
        Self {
            mvp: math::mul(&clip, &model),
            model,
            half: side as f32 / 2.0,
        }
    }

    pub fn project(&self, position: Vec3) -> [f32; 4] {
        let p = math::transform_point(&self.mvp, position);
        let w = self.mvp[3] * position[0]
            + self.mvp[7] * position[1]
            + self.mvp[11] * position[2]
            + self.mvp[15];
        [
            (p[0] + 1.0) * self.half,
            (1.0 - p[1]) * self.half,
            p[2],
            1.0 / w.max(f32::EPSILON),
        ]
    }
}
