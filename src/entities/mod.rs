use bevy::prelude::*;

pub mod chatter;
pub mod crop;
pub mod fruit;
pub mod plugins;
pub mod streamer;

#[derive(Component, Default, PartialEq, Eq)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}
