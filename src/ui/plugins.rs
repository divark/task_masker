use bevy::prelude::*;

use super::chatting::*;
use super::screens::*;
use crate::entities::chatter::speak_to_streamer_from_chatter;
use crate::entities::subscriber::speak_to_streamer_from_subscriber;
use crate::GameState;

#[derive(Default)]
pub struct StartupScreenPlugin;

impl Plugin for StartupScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), spawn_start_screen);
        app.add_systems(OnExit(GameState::Start), despawn_start_screen);

        app.add_systems(OnEnter(GameState::InGame), spawn_ingame_screen);
        app.add_systems(
            OnEnter(GameState::InGame),
            insert_speaker_portrait_background.after(spawn_ingame_screen),
        );

        app.add_systems(
            Update,
            (
                insert_counting_information,
                decrement_health_timer,
                update_healthbar_progress,
                end_ingame_on_no_health,
            )
                .run_if(in_state(GameState::InGame)),
        );

        app.add_systems(OnExit(GameState::InGame), despawn_ingame_screen);

        app.add_systems(OnEnter(GameState::End), spawn_end_screen);
        app.add_systems(OnExit(GameState::End), despawn_end_screen);
        app.add_systems(Update, cycle_screens);
    }
}

#[derive(Default)]
pub struct ChattingPlugin;

impl Plugin for ChattingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Msg>().add_systems(
            Update,
            (
                insert_chatting_information,
                load_msg_into_queue,
                speak_to_streamer_from_chatter,
                speak_to_streamer_from_subscriber,
                load_queued_msg_into_textfield,
                teletype_current_message,
                play_typing_noise,
                activate_waiting_timer,
                unload_msg_on_timeup,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}
