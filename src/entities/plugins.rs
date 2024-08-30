use crate::entities::chatter::*;
use crate::entities::crop::*;
use crate::entities::fruit::*;
use crate::entities::streamer::*;
use crate::entities::subscriber::*;
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
                replace_chatter_tile,
                replace_chatter_sprite,
                trigger_flying_to_streamer,
                fly_to_streamer_to_speak,
                leave_from_streamer_from_chatter,
                return_chatter_to_idle,
                follow_streamer_while_speaking,
                follow_streamer_while_approaching_for_chatter,
            ),
        );
    }
}

#[derive(Default)]
pub struct SubscriberPlugin;

impl Plugin for SubscriberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SubscriberMsg>();
        app.add_systems(
            Update,
            (
                replace_subscriber_sprite,
                replace_subscriber_tile,
                trigger_swimming_to_streamer,
                swim_to_streamer_to_speak,
                leave_from_streamer_from_subscriber,
                return_subscriber_to_idle,
                follow_streamer_while_approaching_for_subscriber,
            ),
        );
    }
}

#[derive(Default)]
pub struct StreamerPlugin;

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OnlineStatus>();
        app.add_systems(
            Update,
            (
                spawn_player_sprite,
                spawn_player_tile,
                move_streamer,
                make_streamer_idle_when_not_moving,
                change_status_for_streamer,
                move_streamer_on_status_change,
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
                replace_fruit_sprites,
                make_fruit_fall,
                make_fruit_dropped,
                pathfind_streamer_to_fruit,
                respawn_fruit,
                drop_random_fruit_on_f_key,
                play_sound_for_fruit,
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
                replace_crop_sprites,
                grow_crop_on_c_key,
                grow_crops,
                pathfind_streamer_to_crops,
                pick_up_crops,
                play_sound_for_crop,
                change_crop_sprite,
            ),
        );
    }
}
