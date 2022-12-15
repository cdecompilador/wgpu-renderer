use crate::model::{ModelUniform, Model};

#[repr(u8)]
pub enum Face {
    UP,
    DOWN
}

pub struct QuadPosition {
    position: cgmath::Vector3<f32>,
    face: Face
}

pub struct QuadRenderer {
    model_uniform: ModelUniform,
    instances: QuadPosition,
}

impl QuadRenderer {
    
}