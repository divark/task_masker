use bevy::prelude::*;

pub mod chatter;
pub mod crop;
pub mod fruit;
pub mod plugins;
pub mod streamer;
pub mod subscriber;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}

#[derive(Component)]
pub struct WaitTimer(pub Timer);
