use bevy::prelude::*;

use super::animations::*;

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
