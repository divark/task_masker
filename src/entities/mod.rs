use std::collections::VecDeque;

use bevy::prelude::*;

pub mod chatter;
pub mod crop;
pub mod fruit;
pub mod plugins;
pub mod streamer;
pub mod subscriber;

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MovementType {
    Walk,
    #[default]
    Fly,
    Swim,
}

#[derive(Component)]
pub struct WaitTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct TriggerQueue(pub VecDeque<()>);
