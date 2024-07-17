mod mock_plugins;

use bevy::prelude::*;
use cucumber::{given, then, when, World};
use mock_plugins::DEFAULT_SUBSCRIBER_SPRITE_IDX;

use crate::mock_plugins::MockPortraitPreferencePlugin;
use task_masker::ui::portrait_preferences::*;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct GamePreferencesWorld {
    pub app: App,

    pub preference_idx: usize,
}

impl GamePreferencesWorld {
    pub fn new() -> Self {
        let mut app = App::new();
        let preference_idx = 0;

        app.add_plugins(MinimalPlugins);

        Self {
            app,
            preference_idx,
        }
    }
}

#[given("a portrait preference recorder,")]
fn load_portrait_preference_recorder(world: &mut GamePreferencesWorld) {
    world.app.add_plugins(MockPortraitPreferencePlugin);
    world.app.update();
}

#[when("I ask for the portrait preference for a user with no preference,")]
fn user_asks_for_preference_without_entry(world: &mut GamePreferencesWorld) {
    let preference_recorder = world
        .app
        .world
        .get_resource::<PortraitPreferences>()
        .expect("user_asks_for_preference_without_entry: PortraitPreferences resource not found.");

    world.preference_idx = preference_recorder.get(String::from("nintend"));
}

#[when("I change a portrait preference for a user,")]
fn user_changes_preference_without_entry(world: &mut GamePreferencesWorld) {
    let mut preference_recorder = world
        .app
        .world
        .get_resource_mut::<PortraitPreferences>()
        .expect("user_changes_preference_without_entry: PortraitPreferences resource not found.");

    preference_recorder.set(String::from("nintend"), 5);
}

#[when("I change the portrait preference again for the user,")]
fn user_changes_preference_with_entry(world: &mut GamePreferencesWorld) {
    let mut preference_recorder = world
        .app
        .world
        .get_resource_mut::<PortraitPreferences>()
        .expect("user_changes_preference_without_entry: PortraitPreferences resource not found.");

    preference_recorder.set(String::from("nintend"), 8);
}

#[then("the index of the default portrait should be returned.")]
fn should_be_default_preference_when_none_exists(world: &mut GamePreferencesWorld) {
    assert_eq!(DEFAULT_SUBSCRIBER_SPRITE_IDX, world.preference_idx);
}

#[then("the portrait preference should be saved for the user.")]
fn user_should_have_preference_saved(world: &mut GamePreferencesWorld) {
    let preference_recorder = world
        .app
        .world
        .get_resource::<PortraitPreferences>()
        .expect("user_should_have_preference_saved: PortraitPreferences resource not found.");

    let expected_portrait_idx = 5;
    world.preference_idx = preference_recorder.get(String::from("nintend"));

    assert_eq!(expected_portrait_idx, world.preference_idx);
}

#[then("the recently changed portrait preference should be saved for the user.")]
fn user_should_have_recent_preference_saved(world: &mut GamePreferencesWorld) {
    let preference_recorder = world
        .app
        .world
        .get_resource::<PortraitPreferences>()
        .expect("user_should_have_preference_saved: PortraitPreferences resource not found.");

    let expected_portrait_idx = 8;
    world.preference_idx = preference_recorder.get(String::from("nintend"));

    assert_eq!(expected_portrait_idx, world.preference_idx);
}

fn main() {
    futures::executor::block_on(GamePreferencesWorld::run(
        "tests/feature-files/portrait-preference.feature",
    ));
}
