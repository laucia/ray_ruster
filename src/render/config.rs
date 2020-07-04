use crate::geometry::types::{Direction, Position};

pub struct CameraConfig {
    pub camera_position: Position,
    pub x: Direction,
    pub y: Direction,
    pub z: Direction,
    pub fov: f64,
    pub aspect_ratio: f64,
    pub width: u32,
    pub height: u32,
}

pub enum NormalMode {
    Phong,
    Triangle,
}

pub struct RenderingConfig {
    pub normal_mode: NormalMode,
}
