use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(debug_ui.in_schedule(OnEnter(GameState::Playing)));
    }
}

fn debug_ui(mut commands: Commands, assets: Res<MyAssets>) {
    println!("Spawning debug UI");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(100.), Val::Px(100.)),
                            border: UiRect::all(Val::Px(2.0)),
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                left: Val::Px(0.),
                                top: Val::Px(0.),
                                ..default()
                            },
                            ..default()
                        },
                        background_color: Color::ALICE_BLUE.into(),
                        ..default()
                    },
                    RelativeCursorPosition::default(),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            position: UiRect {
                                left: Val::Px(0.),
                                top: Val::Px(0.),
                                ..default()
                            },
                            ..default()
                        },
                        text: Text::from_section(
                            "Hello world",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 14.,
                                color: Color::BLACK,
                            },
                        ),
                        ..default()
                    });
                });
        });
}
