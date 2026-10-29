//! A soft radial spot shared by inventory and workspace device stages.
use eframe::egui::{Color32, Painter, Pos2, Shape, Vec2, epaint::Mesh, vec2};

const SEGMENTS: usize = 48;

pub(crate) fn stage_spot(painter: &Painter, center: Pos2, radii: Vec2, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for (scale, ring_color) in [
        (0.45, color.gamma_multiply(0.55)),
        (1.0, Color32::TRANSPARENT),
    ] {
        for i in 0..SEGMENTS {
            let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let offset = vec2(angle.cos() * radii.x, angle.sin() * radii.y) * scale;
            mesh.colored_vertex(center + offset, ring_color);
        }
    }
    let n = SEGMENTS as u32;
    for i in 0..n {
        let j = (i + 1) % n;
        mesh.add_triangle(0, 1 + i, 1 + j);
        mesh.add_triangle(1 + i, 1 + n + i, 1 + n + j);
        mesh.add_triangle(1 + i, 1 + n + j, 1 + j);
    }
    painter.add(Shape::mesh(mesh));
}
