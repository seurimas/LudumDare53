use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(gameplay_ui.in_schedule(OnEnter(GameState::Playing)));
    }
}

pub const ONE_UNIT: f32 = 4.;
pub const FONT_SIZE: f32 = 20.;

fn spawn_stat_block(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    fancy_font: Handle<Font>,
    name: &'static str,
) {
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
                        font: fancy_font.clone(),
                        font_size: FONT_SIZE,
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
                            font_size: FONT_SIZE,
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

fn spawn_stat_ui(parent: &mut ChildBuilder, font: Handle<Font>, fancy_font: Handle<Font>) {
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
            spawn_stat_block(parent, font.clone(), fancy_font.clone(), "Agents");
            spawn_stat_block(parent, font.clone(), fancy_font.clone(), "Corrupted");
            spawn_stat_block(parent, font.clone(), fancy_font.clone(), "Signs");
        });
}

fn spawn_labeled_value(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    label: &'static str,
    name: &'static str,
) {
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
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: FONT_SIZE,
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
                        "?",
                        TextStyle {
                            font: font.clone(),
                            font_size: FONT_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    ..default()
                },
                Name::new(name),
            ));
        });
}

#[derive(Component)]
pub struct ActiveInactiveImages {
    pub active: Handle<Image>,
    pub inactive: Handle<Image>,
    pub deactivated: Handle<Image>,
}

fn spawn_agent_action_button(
    parent: &mut ChildBuilder,
    active_action_button: Handle<Image>,
    inactive_action_button: Handle<Image>,
    deactivated_action_button: Handle<Image>,
    agent_action: AgentAction,
    position: UiRect,
) {
    let images = ActiveInactiveImages {
        active: active_action_button,
        inactive: inactive_action_button.clone(),
        deactivated: deactivated_action_button,
    };
    parent.spawn((
        ButtonBundle {
            style: Style {
                border: UiRect::all(Val::Px(ONE_UNIT)),
                size: Size::new(Val::Px(64.), Val::Px(64.)),
                flex_shrink: 0.,
                position,
                position_type: if position == UiRect::default() {
                    PositionType::default()
                } else {
                    PositionType::Absolute
                },
                ..Default::default()
            },
            image: inactive_action_button.into(),
            ..Default::default()
        },
        agent_action,
        RelativeCursorPosition::default(),
        images,
    ));
}

fn spawn_agent_section(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    action_buttons: HashMap<String, Handle<Image>>,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Name::new("Area Agent"),
        ))
        .with_children(|parent| {
            spawn_labeled_value(parent, font.clone(), "Name", "agent_name");
            spawn_labeled_value(parent, font.clone(), "Power", "agent_power");
            spawn_labeled_value(parent, font.clone(), "Signs", "agent_signs");
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(ONE_UNIT)),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexStart,
                            flex_wrap: FlexWrap::Wrap,
                            gap: Size::all(Val::Px(ONE_UNIT)),
                            size: Size::width(Val::Px(196. + ONE_UNIT * 6.)),
                            ..default()
                        },
                        background_color: Color::RED.into(),
                        ..default()
                    },
                    Name::new("agent_actions"),
                ))
                .with_children(|parent| {
                    spawn_agent_action_button(
                        parent,
                        action_buttons["MoveActive.png"].clone(),
                        action_buttons["Move.png"].clone(),
                        action_buttons["MoveDeactivated.png"].clone(),
                        AgentAction::Move(u32::MAX, u32::MAX, "".to_string()),
                        default(),
                    );
                    spawn_agent_action_button(
                        parent,
                        action_buttons["ProstelytizeActive.png"].clone(),
                        action_buttons["Prostelytize.png"].clone(),
                        action_buttons["ProstelytizeDeactivated.png"].clone(),
                        AgentAction::Prostelytize,
                        default(),
                    );
                    spawn_agent_action_button(
                        parent,
                        action_buttons["BrutalizeActive.png"].clone(),
                        action_buttons["Brutalize.png"].clone(),
                        action_buttons["BrutalizeDeactivated.png"].clone(),
                        AgentAction::Brutalize,
                        default(),
                    );
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(64.), Val::Px(64.)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_agent_action_button(
                                parent,
                                action_buttons["CorruptActive.png"].clone(),
                                action_buttons["Corrupt.png"].clone(),
                                action_buttons["CorruptDeactivated.png"].clone(),
                                AgentAction::Corrupt,
                                UiRect {
                                    left: Val::Px(0.),
                                    right: Val::Px(0.),
                                    top: Val::Px(0.),
                                    bottom: Val::Px(0.),
                                },
                            );
                            spawn_agent_action_button(
                                parent,
                                action_buttons["CorruptAgentActive.png"].clone(),
                                action_buttons["CorruptAgent.png"].clone(),
                                action_buttons["CorruptAgentDeactivated.png"].clone(),
                                AgentAction::CorruptAgent,
                                UiRect {
                                    left: Val::Px(0.),
                                    right: Val::Px(0.),
                                    top: Val::Px(0.),
                                    bottom: Val::Px(0.),
                                },
                            );
                        });
                    spawn_agent_action_button(
                        parent,
                        action_buttons["SacrificeActive.png"].clone(),
                        action_buttons["Sacrifice.png"].clone(),
                        action_buttons["SacrificeDeactivated.png"].clone(),
                        AgentAction::Sacrifice,
                        default(),
                    );
                    spawn_agent_action_button(
                        parent,
                        action_buttons["NextActive.png"].clone(),
                        action_buttons["Next.png"].clone(),
                        action_buttons["NextDeactivated.png"].clone(),
                        AgentAction::None,
                        default(),
                    );
                });
        });
}

fn spawn_area_ui(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    action_buttons: HashMap<String, Handle<Image>>,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            RelativeCursorPosition::default(),
            Name::new("area_ui"),
        ))
        .with_children(|parent| {
            spawn_labeled_value(parent, font.clone(), "", "area_name");
            spawn_labeled_value(parent, font.clone(), "Population", "area_population");
            spawn_labeled_value(parent, font.clone(), "Total Power", "area_total_power");
            spawn_labeled_value(parent, font.clone(), "Followers", "area_followers");
            spawn_labeled_value(parent, font.clone(), "Your Power", "area_your_power");
            spawn_labeled_value(parent, font.clone(), "Corrupted", "area_corrupted");
            spawn_agent_section(parent, font.clone(), action_buttons);
        });
}

fn gameplay_ui(mut commands: Commands, assets: Res<MyAssets>) {
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
            spawn_stat_ui(parent, assets.font.clone(), assets.fancy_font.clone());
            spawn_area_ui(parent, assets.font.clone(), assets.action_buttons.clone());
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
                            font_size: FONT_SIZE,
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
