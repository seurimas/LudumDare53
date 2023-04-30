use crate::prelude::*;

use super::{
    player::GamePlayers,
    turn_ui::TurnReport,
    turns::{apply_turns, Season},
};

pub struct DarknessPlugin;

impl Plugin for DarknessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EvokingState>()
            .add_system(add_turn_end_button.in_schedule(OnEnter(GameState::Playing)))
            .add_system(add_evoking_ui.in_schedule(OnEnter(GameState::Playing)))
            .add_system(evoke_darkness_on_click.run_if(in_state(GameState::Playing)))
            .add_system(watch_evokations.run_if(in_state(GameState::Playing)))
            .add_system(end_evokation.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub enum EvokingState {
    None,
    Evoking {
        evoked: HashMap<PlayerId, (u64, PlayerTurn)>,
    },
    Ready {
        turns: Vec<PlayerTurn>,
        seeds: Vec<u64>,
    },
}

impl Default for EvokingState {
    fn default() -> Self {
        Self::None
    }
}

impl EvokingState {
    pub fn begin(&mut self, player_turn: PlayerTurn) {
        let mut evoked = HashMap::new();
        evoked.insert(
            player_turn.player_id,
            (rand::thread_rng().gen(), player_turn),
        );
        *self = Self::Evoking { evoked };
    }

    pub fn push(&mut self, turn_seed: u64, player_turn: PlayerTurn) {
        if let Self::Evoking { evoked } = self {
            evoked.insert(player_turn.player_id, (turn_seed, player_turn));
        }
    }

    pub fn check(&mut self, players: &GamePlayers) {
        match self {
            Self::Evoking { evoked } => {
                if evoked.len() == players.len() {
                    let mut turns = Vec::new();
                    let mut seeds = Vec::new();
                    for player in players.iter() {
                        let (seed, turn) = evoked.remove(&player.0).unwrap();
                        turns.push(turn);
                        seeds.push(seed);
                    }
                    *self = Self::Ready { turns, seeds };
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evokation {
    pub seed: u64,
    pub player_turn: PlayerTurn,
    pub check_sum: u32,
}

impl Evokation {
    pub fn new(player_turn: PlayerTurn) -> Self {
        let seed = rand::thread_rng().gen();
        let check_sum = player_turn
            .actions
            .iter()
            .fold(0, |acc, (_, action)| acc ^ action.check_sum())
            ^ seed as u32;
        Self {
            seed,
            player_turn,
            check_sum,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.check_sum
            == self
                .player_turn
                .actions
                .iter()
                .fold(0, |acc, (_, action)| acc ^ action.check_sum())
    }

    pub fn retrieve_evokation() -> Option<Evokation> {
        retrieve_from_runes::<Evokation>().filter(|evokation: &Evokation| evokation.is_valid())
    }

    pub fn store_evokation(&self, futhark: bool) -> Option<String> {
        store_in_runes(self, futhark)
    }
}

fn evoke_darkness_on_click(
    mut evoking_state: ResMut<EvokingState>,
    player_turn: Res<PlayerTurn>,
    mut interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<EvokeDarknessButton>,
        ),
    >,
    mut tiles: Query<&mut MapTile>,
    mut evoking_ui: Query<&mut Visibility, With<EvokingUi>>,
    my_assets: Res<MyAssets>,
    audio: Res<Audio>,
) {
    if let Some(interaction) = interaction_query.iter_mut().next() {
        if *interaction == Interaction::Clicked {
            evoking_state.begin(player_turn.clone());
            Evokation::new(player_turn.clone()).store_evokation(true);
            for mut visibility in evoking_ui.iter_mut() {
                *visibility = Visibility::Visible;
            }
            for mut tile in tiles.iter_mut() {
                tile.hovered = false;
                tile.selected = false;
            }
            audio.play(my_assets.evoke_darkness.clone());
        }
    }
}

fn watch_evokations(mut evoking_state: ResMut<EvokingState>) {
    if let Some(evokation) = Evokation::retrieve_evokation() {
        evoking_state.push(evokation.seed, evokation.player_turn);
    }
}

fn end_evokation(
    player_id: Res<PlayerId>,
    game_players: Res<GamePlayers>,
    mut season: ResMut<Season>,
    mut commands: Commands,
    mut player_turn: ResMut<PlayerTurn>,
    mut turn_report: ResMut<TurnReport>,
    mut evoking_state: ResMut<EvokingState>,
    query: Query<&WorldArea>,
    tile_query: Query<(Entity, &MapTile)>,
    mut evoking_ui: Query<&mut Visibility, With<EvokingUi>>,
) {
    evoking_state.check(&game_players);
    if let EvokingState::Ready { turns, seeds } = evoking_state.as_ref() {
        let world_areas = query
            .iter()
            .map(|world_area| world_area.clone())
            .collect::<Vec<WorldArea>>();
        let results = apply_turns(
            **season,
            *player_id,
            turns.clone(),
            seeds.clone(),
            world_areas,
        );
        println!("{:?}", results.report);
        for (entity, map_tile) in tile_query.iter() {
            if let Some(new_world_area) = results.get_new_world_area((map_tile.x, map_tile.y)) {
                commands.entity(entity).insert(new_world_area);
            } else if query.contains(entity) {
                commands.entity(entity).remove::<WorldArea>();
            }
        }
        *turn_report = TurnReport::new(results.report);
        player_turn.reset();
        **season = **season + 1;
        *evoking_state = EvokingState::None;
        for mut visibility in evoking_ui.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Component)]
struct EvokingUi;

fn add_evoking_ui(mut commands: Commands, assets: Res<MyAssets>) {
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
                visibility: Visibility::Hidden,
                ..default()
            },
            EvokingUi,
            RelativeCursorPosition::default(),
        ))
        .with_children(|parent| {
            // Evoking UI
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
                    // Evoking Title
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                size: Size::width(Val::Percent(100.)),
                                ..default()
                            },
                            text: Text::from_section(
                                "Evoking The Darkness",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: FONT_SIZE * 2.,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        Name::new("evoking_title"),
                    ));
                    // Evoking body, explaining that you have the script.
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                flex_grow: 1.,
                                ..default()
                            },
                            text: Text::from_sections(
                                vec![TextSection {
                                    value: "You have evoked the darkness on the third full moon of the season.\n".to_string(),
                                    style: TextStyle {
                                        font: assets.font.clone(),
                                        font_size: FONT_SIZE,
                                        color: Color::WHITE,
                                    },
                                }, TextSection {
                                    value: "You receive a runic script in your mind.\n\n".to_string(),
                                    style: TextStyle {
                                        font: assets.font.clone(),
                                        font_size: FONT_SIZE,
                                        color: Color::WHITE,
                                    },
                                }, TextSection {
                                    value: "Press <C> to copy the script again.".to_string(),
                                    style: TextStyle {
                                        font: assets.font.clone(),
                                        font_size: FONT_SIZE,
                                        color: Color::WHITE,
                                    },
                                }],
                            ),
                            ..default()
                        },
                        Name::new("evoking_body"),
                    ));
                    // Spawn an empty text block, to be filled by a system with players who have not evoked yet.
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                flex_grow: 1.,
                                ..default()
                            },
                            text: Text::from_sections(
                                vec![],
                            ),
                            ..default()
                        },
                        Name::new("evoking_waiting"),
                    ));
                });
        });
}

#[derive(Component)]
struct EvokeDarknessButton;

fn add_turn_end_button(mut commands: Commands, assets: Res<MyAssets>) {
    // Evoke darkness.
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(ONE_UNIT),
                        right: Val::Px(ONE_UNIT),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            EvokeDarknessButton,
            RelativeCursorPosition::default(),
            SimpleTooltip::new(
                "End your turn and evoke the darkness.\n*This will overwrite your clipboard text.*",
            ),
            Name::new("Evoke Darkness"),
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle {
                style: Style {
                    flex_shrink: 1.,
                    ..default()
                },
                text: Text::from_section(
                    "Evoke Darkness",
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
    // Evoke darkness - Inactive.
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(ONE_UNIT),
                        right: Val::Px(ONE_UNIT),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            RelativeCursorPosition::default(),
            SimpleTooltip::new("Assign all agents before ending turn."),
            Name::new("Evoke Darkness Inactive"),
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle {
                style: Style {
                    flex_shrink: 1.,
                    ..default()
                },
                text: Text::from_section(
                    "Evoke Darkness",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 32.,
                        color: Color::BLACK,
                    },
                ),
                background_color: Color::rgba(0.8, 0., 0.9, 0.5).into(),
                ..default()
            },));
        });
}
