#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
struct Appearance {
    color: cgmath::Vector3<f32>,
    origin: cgmath::Vector2<f32>,
    scale: cgmath::Vector2<f32>,
    rotation: f32,
}