use bevy::prelude::*;

pub fn spawn_start_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.0), height: Val::Percent(100.0) },
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::rgb_u8(50, 153, 204).into(),
        ..default()
    };

    let default_font = asset_server.load("font/FiraSans-Bold.ttf");

    let title_section = NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.0), height: Val::Percent(25.0) },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let title_text = TextBundle::from_section("Task Masker",
        TextStyle { font_size: 64.0, font: default_font.clone(), color: Color::BLACK }    
    );

    let button_section = NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.0), height: Val::Percent(25.0) },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let button = ButtonBundle {
        style: Style {
            size: Size { width: Val::Percent(21.1), height: Val::Percent(50.0) },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        //image: todo!(),
        ..default()
    };

    let button_text = TextBundle::from_section("Press Start",
        TextStyle { font: default_font, font_size: 32.0, color: Color::BLACK }
    );

    commands.spawn(background)
        .with_children(|background| {
            background.spawn(title_section)
                .with_children(|title_section| {
                    title_section.spawn(title_text);
                });

            for _i in 0..3 {
                background.spawn(NodeBundle { style: Style { size: Size { width: Val::Percent(100.0), height: Val::Percent(25.0) }, ..default()}, ..default()});
            }
            background.spawn(button_section)
                .with_children(|button_section| {
                    button_section.spawn(button)
                        .with_children(|button| {
                            button.spawn(button_text);
                        });
                });
        });
}