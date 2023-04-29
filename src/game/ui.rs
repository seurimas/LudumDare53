use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(gameplay_ui.in_schedule(OnEnter(GameState::Playing)));
    }
}

const ONE_UNIT: f32 = 4.;

fn spawn_stat_block(parent: &mut ChildBuilder, font: Handle<Font>, name: &'static str) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::GREEN.into(),
                ..default()
            },
            RelativeCursorPosition::default(),
            Name::new(name),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    ..default()
                },
                text: Text::from_section(
                    name,
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.,
                        color: Color::BLACK,
                    },
                ),
                background_color: Color::WHITE.into(),
                ..default()
            });
            parent.spawn((
                TextBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(ONE_UNIT)),
                        ..default()
                    },
                    text: Text::from_section(
                        "0",
                        TextStyle {
                            font: font.clone(),
                            font_size: 14.,
                            color: Color::WHITE,
                        },
                    ),
                    background_color: Color::BLACK.into(),
                    ..default()
                },
                Name::new(format!("{}-value", name)),
            ));
        });
}

fn spawn_stat_ui(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    gap: Size::all(Val::Px(ONE_UNIT)),
                    ..default()
                },
                ..default()
            },
            RelativeCursorPosition::default(),
        ))
        .with_children(|parent| {
            spawn_stat_block(parent, font.clone(), "Agents");
            spawn_stat_block(parent, font.clone(), "Corrupted");
            spawn_stat_block(parent, font.clone(), "Signs");
        });
}

fn spawn_area_label(parent: &mut ChildBuilder, font: Handle<Font>, name: &'static str) {
    parent
        .spawn(NodeBundle {
            style: Style {
                border: UiRect::all(Val::Px(ONE_UNIT)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    ..default()
                },
                text: Text::from_section(
                    name,
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.,
                        color: Color::BLACK,
                    },
                ),
                ..default()
            });
            parent.spawn((
                TextBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(ONE_UNIT)),
                        ..default()
                    },
                    text: Text::from_section(
                        "0",
                        TextStyle {
                            font: font.clone(),
                            font_size: 14.,
                            color: Color::BLACK,
                        },
                    ),
                    ..default()
                },
                Name::new(name),
            ));
        });
}

fn spawn_area_ui(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn(NodeBundle {
            style: Style {
                border: UiRect::all(Val::Px(ONE_UNIT)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            spawn_area_label(parent, font.clone(), "Area Name");
            spawn_area_label(parent, font.clone(), "Followers");
            spawn_area_label(parent, font.clone(), "Power");
            spawn_area_label(parent, font.clone(), "Corrupted");
        });
}

fn gameplay_ui(mut commands: Commands, assets: Res<MyAssets>) {
    println!("Spawning debug UI");
    // Stats area.
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            spawn_stat_ui(parent, assets.font.clone());
            spawn_area_ui(parent, assets.font.clone());
        });
    // Tooltip.
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(ONE_UNIT * 2.),
                        ..default()
                    },
                    size: Size::width(Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            Name::new("tooltip_parent"),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(ONE_UNIT)),
                        flex_shrink: 1.,
                        ..default()
                    },
                    text: Text::from_section(
                        "Tooltip",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 14.,
                            color: Color::BLACK,
                        },
                    ),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                Name::new("tooltip"),
            ));
        });
}
