use bevy::prelude::{default, Component};

pub mod streamer;

#[derive(Component, Default)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}
