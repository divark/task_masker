use crate::entities::crop::*;
use crate::entities::fruit::*;
use crate::{spawn_player, GameState};
use bevy::prelude::*;

#[derive(Default)]
pub struct StreamerPlugin;

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_player).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Default)]
pub struct FruitPlugin;

impl Plugin for FruitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                replace_fruit_tiles,
                make_fruit_fall,
                make_fruit_dropped,
                pathfind_streamer_to_fruit,
                mark_crop_tile_visited,
                claim_fruit_from_streamer,
                respawn_fruit,
                drop_random_fruit_on_f_key,
            ),
        );
    }
}

#[derive(Default)]
pub struct CropPlugin;

impl Plugin for CropPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewSubscriber>();
        app.add_systems(
            Update,
            (
                replace_crop_tiles,
                grow_crop_on_c_key,
                grow_crops,
                inform_streamer_of_grown_crops,
                pick_up_crops,
            ),
        );
    }
}
