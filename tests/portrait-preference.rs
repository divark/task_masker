mod mock_plugins;

use bevy::prelude::*;
use cucumber::{given, then, when, World};

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

#[then("the index of the default portrait should be returned.")]
fn should_be_default_preference_when_none_exists(world: &mut GamePreferencesWorld) {
    let default_portrait_idx = 0;
    assert_eq!(default_portrait_idx, world.preference_idx);
}

fn main() {
    futures::executor::block_on(GamePreferencesWorld::run(
        "tests/feature-files/portrait-preference.feature",
    ));
}
