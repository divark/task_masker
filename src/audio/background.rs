use bevy::audio::Volume;
use bevy::{audio::PlaybackMode, prelude::*};
use rand::seq::SliceRandom;

use crate::ui::screens::ScreenLabel;

#[derive(Component)]
pub struct Music;

#[derive(Component)]
pub struct SoundTrack(Vec<String>);

pub fn insert_background_noises(asset_loader: Res<AssetServer>, mut commands: Commands) {
    let startup_sounds = AudioBundle {
        source: asset_loader.load("music/start_screen.wav"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.25),
            ..default()
        },
    };

    commands.spawn(startup_sounds);
}

pub fn remove_music(mut commands: Commands, music_query: Query<(Entity, &AudioSink), With<Music>>) {
    if music_query.is_empty() {
        return;
    }

    for (music_entity, music_control) in music_query.iter() {
        music_control.stop();
        commands.entity(music_entity).despawn_recursive();
    }
}

const NUM_TRACKS: u32 = 12;

pub fn add_soundtrack(mut commands: Commands) {
    let mut music_files = Vec::new();
    for i in 1..=NUM_TRACKS {
        music_files.push(format!("music/{}.WAV", i));
    }

    let soundtrack = SoundTrack(music_files);

    commands.spawn(soundtrack);
}

pub fn remove_soundtrack(
    mut commands: Commands,
    soundtrack_query: Query<Entity, With<SoundTrack>>,
) {
    if soundtrack_query.is_empty() {
        return;
    }

    let soundtrack_entity = soundtrack_query
        .get_single()
        .expect("Soundtrack should exist.");
    commands.entity(soundtrack_entity).despawn_recursive();
}

pub fn randomly_choose_song(
    screen_ui: Query<&ScreenLabel>,
    soundtrack_query: Query<&SoundTrack>,
    music_query: Query<&Music>,
    asset_loader: Res<AssetServer>,
    mut commands: Commands,
) {
    if screen_ui.is_empty() || soundtrack_query.is_empty() || !music_query.is_empty() {
        return;
    }

    let screen_label = screen_ui
        .get_single()
        .expect("Some screen should exist by now.");
    if *screen_label != ScreenLabel::InGame {
        return;
    }

    let soundtrack = soundtrack_query
        .get_single()
        .expect("Soundtrack should exist by now.");
    let chosen_track = soundtrack
        .0
        .choose(&mut rand::thread_rng())
        .expect("An option of track should exist.");
    let music = AudioBundle {
        source: asset_loader.load(chosen_track),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    };

    commands.spawn((music, Music));
}
