use bevy::prelude::*;

pub mod streamer;

#[derive(Component, Default)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}
