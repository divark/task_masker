use bevy::prelude::*;

pub mod fruit;
pub mod plugins;
pub mod streamer;

#[derive(Component, Default)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}
