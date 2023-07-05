use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub enum ScreenLabel {
    Start,
    InGame,
    End,
}

pub fn spawn_start_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        //background_color: Color::rgb_u8(50, 153, 204).into(),
        ..default()
    };

    let default_font = asset_server.load("font/FiraSans-Bold.ttf");

    let title_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(25.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let title_text = TextBundle::from_section(
        "Task Masker",
        TextStyle {
            font_size: 64.0,
            font: default_font.clone(),
            color: Color::BLACK,
        },
    );

    let button_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(25.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let button_image = asset_server.load("ui/flat_button_start.png");

    let button = ButtonBundle {
        style: Style {
            size: Size {
                width: Val::Percent(21.1),
                height: Val::Percent(50.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        image: UiImage::new(button_image),
        ..default()
    };

    let button_text = TextBundle::from_section(
        "Press Enter",
        TextStyle {
            font: default_font,
            font_size: 32.0,
            color: Color::BLACK,
        },
    );

    commands
        .spawn((background, ScreenLabel::Start))
        .with_children(|background| {
            background
                .spawn(title_section)
                .with_children(|title_section| {
                    title_section.spawn(title_text);
                });

            for _i in 0..3 {
                background.spawn(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Percent(100.0),
                            height: Val::Percent(25.0),
                        },
                        ..default()
                    },
                    ..default()
                });
            }
            background
                .spawn(button_section)
                .with_children(|button_section| {
                    button_section.spawn(button).with_children(|button| {
                        button.spawn(button_text);
                    });
                });
        });
}

pub fn despawn_start_screen(
    mut commands: Commands,
    start_screen_items: Query<(Entity, &ScreenLabel)>,
) {
    for (start_screen_item, label) in start_screen_items.iter() {
        if let ScreenLabel::Start = label {
            commands.entity(start_screen_item).despawn_recursive();
        }
    }
}

pub fn spawn_ingame_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Start,
            ..default()
        },
        //background_color: Color::rgb_u8(50, 153, 204).into(),
        ..default()
    };

    let default_font = asset_server.load("font/FiraSans-Bold.ttf");

    let title_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(50.0),
                height: Val::Percent(8.3),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: Color::GREEN.into(),
        ..default()
    };

    let title_text = TextBundle::from_section(
        "12345/12345",
        TextStyle {
            font_size: 32.0,
            font: default_font,
            color: Color::BLACK,
        },
    );

    let speaker_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(50.0),
                height: Val::Percent((120.0 / 720.0) * 100.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let speaker_portrait_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent((120.0 / 640.0) * 100.0),
                height: Val::Percent(100.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let speaker_portrait = ImageBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            ..default()
        },
        image: UiImage::new(asset_server.load("caveman/portrait.png")),
        ..default()
    };

    let dialogue_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: Color::WHITE.into(),
        ..default()
    };

    commands
        .spawn((background, ScreenLabel::InGame))
        .with_children(|background| {
            background
                .spawn(title_section)
                .with_children(|title_section| {
                    title_section.spawn(title_text);
                });

            background
                .spawn(speaker_section)
                .with_children(|speaker_section| {
                    speaker_section
                        .spawn(speaker_portrait_section)
                        .with_children(|speaker_portrait_section| {
                            speaker_portrait_section.spawn(speaker_portrait);
                        });

                    speaker_section.spawn(dialogue_section);
                });
        });
}

pub fn despawn_ingame_screen(mut commands: Commands, ingame_screen_items: Query<(Entity, &ScreenLabel)>) {
    for (ingame_screen_item, label) in ingame_screen_items.iter() {
        if let ScreenLabel::InGame = label {
            commands.entity(ingame_screen_item).despawn_recursive();
        }
    }
}

pub fn spawn_end_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        //background_color: Color::rgb_u8(50, 153, 204).into(),
        ..default()
    };

    let default_font = asset_server.load("font/FiraSans-Bold.ttf");

    let title_section = NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(25.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let title_text = TextBundle::from_section(
        "Thanks for Watching!",
        TextStyle {
            font_size: 64.0,
            font: default_font,
            color: Color::BLACK,
        },
    );

    commands
        .spawn((background, ScreenLabel::End))
        .with_children(|background| {
            background
                .spawn(title_section)
                .with_children(|title_section| {
                    title_section.spawn(title_text);
                });
        });
}

pub fn despawn_end_screen(mut commands: Commands, end_screen_items: Query<(Entity, &ScreenLabel)>) {
    for (end_screen_item, label) in end_screen_items.iter() {
        if let ScreenLabel::End = label {
            commands.entity(end_screen_item).despawn_recursive();
        }
    }
}

pub fn cycle_screens(
    keyboard_input: Res<Input<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Return) {
        return;
    }

    match &game_state.0 {
        GameState::Start => next_state.set(GameState::InGame),
        GameState::InGame => next_state.set(GameState::End),
        GameState::End => next_state.set(GameState::Start),
    }
}
