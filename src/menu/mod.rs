use bevy::{input::keyboard::KeyboardInput, utils::HashSet};

use crate::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_system(add_welcome_screen.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(add_name_screen.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(add_new_game_screen.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(remove_main_menu.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(handle_button_clicks.run_if(in_state(GameState::MainMenu)))
            .add_system(handle_typing.run_if(in_state(GameState::MainMenu)))
            .add_system(watch_for_players.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Debug)]
pub enum MainMenu {
    Welcome,
    EnterName,
    NewGame,
    LoadGame,
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub name: String,
    pub awaiting_name: bool,
    pub awaiting_players: bool,
    pub players: HashSet<Joiner>,
    pub ai: usize,
}

fn add_welcome_screen(mut commands: Commands, assets: Res<MyAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(0.),
                        bottom: Val::Px(0.),
                    },
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            MainMenu::Welcome,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    position: UiRect {
                        top: Val::Px(ONE_UNIT),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::from_sections(vec![
                    TextSection {
                        value: "Welcome to ".to_string(),
                        style: TextStyle {
                            font: assets.font.clone(),
                            font_size: 40.,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "Signs of Corruption".to_string(),
                        style: TextStyle {
                            font: assets.font.clone(),
                            font_size: 40.,
                            color: Color::rgb(1., 0.5, 1.),
                        },
                    },
                ]),
                ..Default::default()
            });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(ONE_UNIT)),
                            size: Size::width(Val::Percent(50.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    MainMenuElement::StartNewGame,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "New Game",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(ONE_UNIT)),
                            size: Size::width(Val::Percent(50.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    MainMenuElement::LoadGame,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Load Game",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ),
                        ..Default::default()
                    });
                });
        });
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct Joiner {
    name: String,
    seed: u32,
}

fn add_new_game_screen(mut commands: Commands, assets: Res<MyAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(0.),
                        bottom: Val::Px(0.),
                    },
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            MainMenu::NewGame,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_sections(vec![
                    TextSection {
                        value: "You receive a runic script in your mind.\n\n".to_string(),
                        style: TextStyle {
                            font: assets.font.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "Press <C> to copy the script again.".to_string(),
                        style: TextStyle {
                            font: assets.font.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                        },
                    },
                ]),
                ..Default::default()
            });
            parent.spawn((
                TextBundle {
                    text: Text::from_sections(vec![
                        TextSection {
                            value: "Players in this game:\n".to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: "____".to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: "0 AI Players".to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value:
                                "Left click to add an AI player. Right click to remove an AI player."
                                    .to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value:
                                "Press enter to begin the game. Press escape to return to the main menu."
                                    .to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                    ]),
                    ..Default::default()
                },
                MainMenuElement::PlayerList,
            ));
        });
}

fn add_name_screen(mut commands: Commands, assets: Res<MyAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            MainMenu::EnterName,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_sections(vec![
                        TextSection {
                            value: "Type your player name, then press enter:\n".to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: "_".to_string(),
                            style: TextStyle {
                                font: assets.font.clone(),
                                font_size: FONT_SIZE,
                                color: Color::WHITE,
                            },
                        },
                    ]),
                    ..Default::default()
                },
                MainMenuElement::NameField,
            ));
        });
}

fn remove_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component, PartialEq)]
enum MainMenuElement {
    StartNewGame,
    NameField,
    ConfirmPlayers,
    PlayerList,
    LoadGame,
}
fn watch_for_players(
    mut commands: Commands,
    mut menu_state: ResMut<MenuState>,
    mut text: Query<(&MainMenuElement, &mut Text)>,
    mut mouse_input: Res<Input<MouseButton>>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut menus: Query<(&MainMenu, &mut Visibility)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !menu_state.awaiting_players {
        return;
    } else if menu_state.awaiting_name {
        // Wait a frame.
        menu_state.awaiting_name = false;
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        menu_state.ai += 1;
    } else if mouse_input.just_pressed(MouseButton::Right) {
        menu_state.ai = menu_state.ai.saturating_sub(1);
    }
    if keyboard_input.just_pressed(KeyCode::Return) {
        let players: Vec<String> = menu_state
            .players
            .iter()
            .map(|joiner| joiner.name.to_string())
            .collect();
        let game_players = GamePlayers::new(players.clone());
        let my_player = *game_players
            .iter()
            .find(|(_, p)| p.eq_ignore_ascii_case(&menu_state.name))
            .unwrap()
            .0;
        commands.insert_resource(generate_map(game_players.keys().cloned().collect()));
        commands.insert_resource(PlayerTurn::new(my_player));
        commands.insert_resource(my_player);
        commands.insert_resource(game_players);
        commands.insert_resource(MenuState::default());
        commands.insert_resource(Season(1));
        next_state.set(GameState::Playing);
    }
    for (element, mut text) in text.iter_mut() {
        if *element == MainMenuElement::PlayerList {
            text.sections[2].value = format!("{} AI Players", menu_state.ai);
        }
    }
    if let Some(joiner) = retrieve_from_runes::<Joiner>() {
        if menu_state.players.contains(&joiner) {
            return;
        } else {
            menu_state.players.insert(joiner.clone());
            for (element, mut text) in text.iter_mut() {
                if *element == MainMenuElement::PlayerList {
                    text.sections[1]
                        .value
                        .push_str(&format!("{}\n", joiner.name));
                }
            }
        }
    }
}

fn handle_typing(
    keyboard_input: Res<Input<KeyCode>>,
    mut events: EventReader<ReceivedCharacter>,
    mut menu_state: ResMut<MenuState>,
    mut menus: Query<(&MainMenu, &mut Visibility)>,
    mut text: Query<(&MainMenuElement, &mut Text)>,
) {
    if menu_state.awaiting_name {
        let mut name_changed = false;
        if keyboard_input.just_pressed(KeyCode::Return) {
            let name = menu_state.name.trim().to_string();
            let joiner = Joiner {
                name: name.clone(),
                seed: rand::thread_rng().gen(),
            };

            menu_state.players = HashSet::new();
            menu_state.players.insert(joiner.clone());
            menu_state.awaiting_players = true;

            for (element, mut text) in text.iter_mut() {
                if *element == MainMenuElement::PlayerList {
                    text.sections[1].value = format!("{}\n", name);
                }
            }

            store_in_runes(joiner, true);
            switch_menu(MainMenu::NewGame, &mut menus);
        } else if keyboard_input.just_pressed(KeyCode::Escape) {
            menu_state.awaiting_name = false;
            for (menu, mut visibility) in menus.iter_mut() {
                if *menu == MainMenu::Welcome {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        } else if keyboard_input.just_pressed(KeyCode::Back) {
            menu_state.name.pop();
            name_changed = true;
        } else {
            for event in events.iter() {
                menu_state.name.push(event.char);
                name_changed = true;
            }
        }
        if name_changed {
            for (element, mut text) in text.iter_mut() {
                if *element == MainMenuElement::NameField {
                    text.sections[1].value = menu_state.name.clone();
                }
            }
        }
    }
}

fn switch_menu(new_menu: MainMenu, menus: &mut Query<(&MainMenu, &mut Visibility)>) {
    for (menu, mut visibility) in menus.iter_mut() {
        if *menu == new_menu {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn handle_button_clicks(
    mut menu_state: ResMut<MenuState>,
    interactions: Query<(&MainMenuElement, &Interaction), Changed<Interaction>>,
    mut menus: Query<(&MainMenu, &mut Visibility)>,
) {
    for (button, interaction) in interactions.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                MainMenuElement::StartNewGame => {
                    menu_state.awaiting_name = true;
                    switch_menu(MainMenu::EnterName, &mut menus);
                }
                MainMenuElement::LoadGame => {
                    // TODO
                }
                _ => {}
            }
        }
    }
}
