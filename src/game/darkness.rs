use crate::prelude::*;

use super::multiplayer::{generate_runes, parse_runes};

pub struct DarknessPlugin;

impl Plugin for DarknessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EvokingState>()
            .add_system(add_evoking_ui.in_schedule(OnEnter(GameState::Playing)))
            .add_system(debug_evokations.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub enum EvokingState {
    None,
    Evoking {
        evoked: HashMap<PlayerId, PlayerTurn>,
    },
    Ready,
}

impl Default for EvokingState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evokation {
    pub player_turn: PlayerTurn,
    pub check_sum: u32,
}

impl Evokation {
    pub fn new(player_turn: PlayerTurn) -> Self {
        let check_sum = player_turn
            .actions
            .iter()
            .fold(0, |acc, (_, action)| acc ^ action.check_sum());
        Self {
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
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.get_text())
            .ok()
            .map(|text| {
                let futhark = parse_runes(&text, true);
                if futhark.len() > 0 {
                    futhark
                } else {
                    parse_runes(&text, false)
                }
            })
            .and_then(|data| postcard::from_bytes(data.as_slice()).ok())
            .filter(|evokation: &Evokation| evokation.is_valid())
    }

    pub fn store_evokation(&self, futhark: bool) -> Option<String> {
        let data = postcard::to_allocvec(self).unwrap();
        let runes = generate_runes(data.as_slice(), futhark);
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(runes.clone()))
            .ok()
            .map(|_| runes)
    }
}

impl EvokingState {
    pub fn begin(&mut self, player_turn: PlayerTurn) {
        let mut evoked = HashMap::new();
        evoked.insert(player_turn.player_id, player_turn);
        *self = Self::Evoking { evoked };
    }
}

fn debug_evokations(player_turn: Res<PlayerTurn>) {
    if let Some(evokation) = Evokation::retrieve_evokation() {
        // println!("Evokation: {:?}", evokation);
    } else {
        // Evokation::new(player_turn.clone()).store_evokation(true);
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
