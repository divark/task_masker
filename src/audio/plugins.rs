use bevy::prelude::*;

use super::background::*;
use crate::GameState;

#[derive(Default)]
pub struct BackgroundMusicPlugin;

impl Plugin for BackgroundMusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_background_noises)
            .add_systems(Update, randomly_choose_song)
            .add_systems(OnEnter(GameState::Start), add_soundtrack)
            .add_systems(OnExit(GameState::End), remove_music)
            .add_systems(OnExit(GameState::InGame), (remove_soundtrack, remove_music));
    }
}
