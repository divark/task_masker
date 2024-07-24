use bevy::prelude::States;

pub mod audio;
pub mod entities;
pub mod map;
pub mod ui;
pub mod visual;

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum GameState {
    #[default]
    Start,
    InGame,
    End,
}
