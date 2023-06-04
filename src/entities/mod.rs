use bevy::prelude::Component;

pub mod streamer;

#[derive(Component)]
pub enum MovementType {
    Walk,
    Fly,
    Swim,
}
