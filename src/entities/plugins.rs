use crate::entities::chatter::*;
use crate::entities::crop::*;
use crate::entities::fruit::*;
use crate::entities::streamer::*;
use crate::GameState;
use bevy::prelude::*;

#[derive(Default)]
pub struct ChatterPlugin;

impl Plugin for ChatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMsg>();
        app.add_systems(
            Update,
            (
                replace_chatter,
                trigger_flying_to_streamer,
                fly_to_streamer_to_speak,
                speak_to_streamer,
                leave_from_streamer,
                follow_streamer_while_speaking,
                follow_streamer_while_approaching,
            ),
        );
    }
}

#[derive(Default)]
pub struct StreamerPlugin;

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player,
                move_streamer,
                move_streamer_on_spacebar,
                test_streamer_msg,
                queue_destination_for_streamer,
                update_status_when_speaking,
            )
                .run_if(in_state(GameState::InGame)),
        );
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
                pathfind_streamer_to_crops,
                pick_up_crops,
            ),
        );
    }
}
