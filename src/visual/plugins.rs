use bevy::prelude::*;

use super::animations::*;
use super::environment::*;

#[derive(Default)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                insert_animation_information,
                change_sprite_direction,
                change_fish_or_chatter_direction,
                animate,
                check_if_idle,
            ),
        );
    }
}

#[derive(Default)]
pub struct EnvironmentAnimationsPlugin;

impl Plugin for EnvironmentAnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                replace_campfire_tile,
                replace_campfire_sprite,
                make_streamer_face_campfire,
            ),
        );
    }
}
