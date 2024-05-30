use bevy::prelude::*;

use task_masker::entities::chatter::*;
use task_masker::entities::streamer::*;
use task_masker::map::tiled::*;
use task_masker::ui::chatting::Msg;

#[derive(Default)]
pub struct MockChatterPlugin;

impl Plugin for MockChatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMsg>();
        app.add_event::<Msg>();
        app.add_systems(
            Update,
            (
                replace_chatter_tile,
                fly_to_streamer_to_speak.after(replace_chatter_tile),
                speak_to_streamer_from_chatter.after(fly_to_streamer_to_speak),
                leave_from_streamer_from_chatter.after(speak_to_streamer_from_chatter),
                return_chatter_to_idle,
                follow_streamer_while_speaking,
                follow_streamer_while_approaching_for_chatter,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockStreamerPlugin;

impl Plugin for MockStreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_tile,
                move_streamer,
                queue_destination_for_streamer.after(spawn_player_tile),
                make_streamer_idle_when_not_moving,
                update_status_when_speaking,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockTiledMapPlugin;

impl Plugin for MockTiledMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tiles_from_tiledmap);
    }
}