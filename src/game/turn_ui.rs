use bevy::utils::HashMap;

use crate::prelude::*;

pub struct TurnUiPlugin;

impl Plugin for TurnUiPlugin {
    fn build(&self, app: &mut App) {
        let mut debug_players = HashMap::default();
        debug_players.insert(PlayerId(0), "Player 1".to_string());
        app.insert_resource(PlayerTurn::new(PlayerId(0), 5))
            .insert_resource(PlayerId(0))
            .insert_resource(GamePlayers(debug_players))
            .insert_resource(Season(0))
            .init_resource::<TurnReport>()
            .add_system(update_end_turn_button.run_if(in_state(GameState::Playing)))
            .add_system(review_turn_on_click.run_if(in_state(GameState::Playing)))
            .add_system(update_review_turn_button.run_if(in_state(GameState::Playing)))
            .add_system(add_review_button.in_schedule(OnEnter(GameState::Playing)))
            .add_system(add_turn_report_ui.in_schedule(OnEnter(GameState::Playing)))
            .add_system(view_turn_report.run_if(in_state(GameState::Playing)))
            .add_system(hide_turn_report.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, Clone)]
pub enum TurnReportEvent {
    GameStart {
        player_names: Vec<String>,
    },
    AgentAction {
        location: (i32, i32),
        location_name: String,
        agent_name: String,
        action: AgentAction,
        success_amount: u32,
        fail_amount: u32,
    },
    Brutalized {
        location: (i32, i32),
        location_name: String,
        followers: u32,
        locals: u32,
    },
    Sacrificed {
        location: (i32, i32),
        location_name: String,
        follower: bool,
    },
    SignSeen {
        location: (i32, i32),
        location_name: String,
        mine: bool,
    },
    GameOver {
        winner: PlayerId,
        scores: HashMap<PlayerId, u32>,
    },
    NewTurn {
        turn: i32,
    },
}

impl TurnReportEvent {
    fn get_title(&self) -> String {
        match self {
            TurnReportEvent::GameStart { .. } => "Campaign Start".to_string(),
            TurnReportEvent::AgentAction {
                location_name,
                agent_name,
                action,
                ..
            } => match action {
                AgentAction::Brutalize => format!("Brutality at {}", location_name),
                AgentAction::Corrupt => format!("Corruption at {}", location_name),
                AgentAction::Sacrifice => format!("Sacrifice at {}", location_name),
                AgentAction::Prostelytize => format!("Prostelytizing at {}", location_name),
                AgentAction::Move(_) => format!("{} arrived at {}", agent_name, location_name),
                AgentAction::None => format!("???"),
            },
            TurnReportEvent::Brutalized { location_name, .. } => {
                format!("Violence erupts at {}!", location_name,)
            }
            TurnReportEvent::Sacrificed {
                location_name,
                follower,
                ..
            } => {
                if *follower {
                    format!("Follower sacrificed at {}!", location_name)
                } else {
                    format!("Body found at {}!", location_name)
                }
            }
            TurnReportEvent::SignSeen {
                location_name,
                mine,
                ..
            } => {
                if *mine {
                    format!("Sign of Corruption at {}!", location_name)
                } else {
                    format!("Strangeness at {}!", location_name)
                }
            }
            TurnReportEvent::GameOver { winner, .. } => format!("Game Over"),
            TurnReportEvent::NewTurn { turn } => format!("New Season"),
        }
    }

    fn get_sections(&self) -> Vec<String> {
        match self {
            TurnReportEvent::GameStart { player_names } => vec![format!(
                "The campaign has begun! {} players have joined.",
                player_names.len()
            )],
            TurnReportEvent::AgentAction {
                location_name,
                agent_name,
                action,
                success_amount,
                fail_amount,
                ..
            } => match action {
                AgentAction::Brutalize => vec![
                    format!(
                        "{} arrived at {} and brutalized the locals.\n",
                        agent_name, location_name
                    ),
                    format!("{} locals were killed or forced to flee.\n", fail_amount),
                    format!("{} followers were gained.\n", success_amount),
                ],
                AgentAction::Corrupt => vec![format!(
                    "{} arrived at {} and corrupted the locals.",
                    agent_name, location_name
                )],
                AgentAction::Sacrifice => vec![format!(
                    "{} arrived at {} and sacrificed a follower.",
                    agent_name, location_name
                )],
                AgentAction::Prostelytize => vec![format!(
                    "{} arrived at {} and converted the locals.",
                    agent_name, location_name
                )],
                AgentAction::Move(_) => {
                    vec![format!("{} arrived at {}.", agent_name, location_name)]
                }
                AgentAction::None => vec![format!("???")],
            },
            TurnReportEvent::Brutalized {
                location_name,
                followers,
                locals,
                ..
            } => vec![format!(
                "Brutality at {}! {} followers and {} locals were killed.",
                location_name, followers, locals
            )],
            TurnReportEvent::Sacrificed {
                location_name,
                follower,
                ..
            } => {
                if *follower {
                    vec![format!("A follower was sacrificed at {}.", location_name)]
                } else {
                    vec![format!("A sacrifice was made at {}.", location_name)]
                }
            }
            TurnReportEvent::SignSeen {
                location_name,
                mine,
                ..
            } => {
                if *mine {
                    vec![format!(
                        "A sign of corruption was seen at {}.",
                        location_name
                    )]
                } else {
                    vec![format!("Something strange was seen at {}.", location_name)]
                }
            }
            TurnReportEvent::GameOver { winner, scores } => vec![format!(
                "Game Over! Player {} wins with {} points!",
                winner.0,
                scores.get(winner).unwrap()
            )],
            TurnReportEvent::NewTurn { turn } => vec![
                format!("A new season begins!"),
                format!(
                    "It has been {} seasons since your campaign has begun.",
                    turn
                ),
            ],
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct TurnReport {
    pub events: Vec<TurnReportEvent>,
    pub event_id: Option<usize>,
    pub rendered_event_id: Option<usize>,
}

impl TurnReport {
    pub fn new(events: Vec<TurnReportEvent>) -> Self {
        Self {
            events,
            event_id: Some(0),
            rendered_event_id: None,
        }
    }
}

fn hide_turn_report(
    turn_report: Res<TurnReport>,
    mut report_query: Query<&mut Visibility, With<TurnReportUi>>,
) {
    if turn_report.event_id.is_none() {
        for mut visibility in report_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    } else {
        for mut visibility in report_query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }
}

fn view_turn_report(
    mut turn_report: ResMut<TurnReport>,
    keyboard: Res<Input<KeyCode>>,
    mut text_query: Query<(&Name, &mut Text, &mut Visibility)>,
    assets: Res<MyAssets>,
) {
    if let Some(mut event_id) = turn_report.event_id {
        if keyboard.just_pressed(KeyCode::Space) {
            // Advance the turn report.
            event_id = event_id + 1;
            println!("Advancing turn report to {}.", event_id);
        } else if keyboard.just_pressed(KeyCode::Back) && event_id > 0 {
            // Go back in the turn report.
            event_id = event_id - 1;
            println!("Going back in turn report to {}.", event_id);
        } else if keyboard.just_pressed(KeyCode::Escape) {
            // Close the turn report.
            println!("Closing turn report.");
            turn_report.event_id = None;
        }
        turn_report.event_id = Some(event_id);
        if event_id >= turn_report.events.len() {
            turn_report.event_id = None;
        } else if turn_report.rendered_event_id != turn_report.event_id {
            turn_report.rendered_event_id = turn_report.event_id;

            let event = turn_report.events.get(event_id).unwrap();

            if let Some(mut title) = text_query
                .iter_mut()
                .find(|(name, _, _)| name.eq_ignore_ascii_case("turn_report_title"))
            {
                title.1.sections[0].value = event.get_title();
            }
            if let Some(mut body) = text_query
                .iter_mut()
                .find(|(name, _, _)| name.eq_ignore_ascii_case("turn_report_body"))
            {
                body.1.sections = event
                    .get_sections()
                    .drain(..)
                    .map(|s| TextSection {
                        value: s,
                        style: TextStyle {
                            font: assets.font.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                        },
                    })
                    .collect();
            }
        }
    }
}

fn update_end_turn_button(
    player_turn: Res<PlayerTurn>,
    mut button_query: Query<(&Name, &mut Visibility)>,
) {
    if player_turn.get_unassigned_agents() > 0 {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Evoke Darkness") {
                *visibility = Visibility::Hidden;
            } else if name.eq_ignore_ascii_case("Evoke Darkness Inactive") {
                *visibility = Visibility::Visible;
            }
        });
    } else {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Evoke Darkness") {
                *visibility = Visibility::Visible;
            } else if name.eq_ignore_ascii_case("Evoke Darkness Inactive") {
                *visibility = Visibility::Hidden;
            }
        });
    }
}

fn update_review_turn_button(
    turn_report: Res<TurnReport>,
    mut button_query: Query<(&Name, &mut Visibility)>,
) {
    if turn_report.event_id.is_none() && turn_report.events.len() > 0 {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Review Turn") {
                *visibility = Visibility::Visible;
            }
        });
    } else {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Review Turn") {
                *visibility = Visibility::Hidden;
            }
        });
    }
}

fn review_turn_on_click(
    mut turn_report: ResMut<TurnReport>,
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<ReviewTurnButton>),
    >,
) {
    if let Some(interaction) = interaction_query.iter_mut().next() {
        if *interaction == Interaction::Clicked {
            turn_report.event_id = Some(0);
        }
    }
}

#[derive(Component)]
pub struct TurnReportUi;

fn add_turn_report_ui(mut commands: Commands, assets: Res<MyAssets>) {
    // Center me.
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(0.),
                        bottom: Val::Px(0.),
                    },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            TurnReportUi,
            RelativeCursorPosition::default(),
        ))
        .with_children(|parent| {
            // Turn report window.
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(500.), Val::Px(500.)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.5).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Turn report title.
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                size: Size::width(Val::Percent(100.)),
                                ..default()
                            },
                            text: Text::from_section(
                                "Turn Report",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE * 2.,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("turn_report_title"),
                    ));
                    // Turn report body.
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                flex_grow: 1.,
                                ..default()
                            },
                            text: Text::from_section(
                                "This is a test, this is only a test",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("turn_report_body"),
                    ));
                    // Helper - Back
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    bottom: Val::Px(ONE_UNIT),
                                    left: Val::Px(ONE_UNIT),
                                    ..default()
                                },
                                ..default()
                            },
                            text: Text::from_section(
                                "Previous: Backspace",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("turn_report_back"),
                    ));
                    // Helper - Exit
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    bottom: Val::Px(ONE_UNIT),
                                    ..default()
                                },
                                ..default()
                            },
                            text: Text::from_section(
                                "Exit: ESC",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("turn_report_esc"),
                    ));
                    // Helper - Forward
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    bottom: Val::Px(ONE_UNIT),
                                    right: Val::Px(ONE_UNIT),
                                    ..default()
                                },
                                ..default()
                            },
                            text: Text::from_section(
                                "Next: Space",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("turn_report_back"),
                    ));
                });
        });
}

#[derive(Component)]
struct ReviewTurnButton;

fn add_review_button(mut commands: Commands, assets: Res<MyAssets>) {
    // Review turn.
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(ONE_UNIT),
                        left: Val::Px(ONE_UNIT),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            ReviewTurnButton,
            RelativeCursorPosition::default(),
            Name::new("Review Turn"),
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle {
                style: Style {
                    flex_shrink: 1.,
                    ..default()
                },
                text: Text::from_section(
                    "Review The Evoking",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 32.,
                        color: Color::BLACK,
                    },
                ),
                background_color: Color::rgb(0.8, 0., 0.9).into(),
                ..default()
            },));
        });
}
