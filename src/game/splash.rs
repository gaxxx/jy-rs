use std::collections::HashMap;

use lazy_static::lazy_static;

use bevy::app::AppExit;
use bevy::prelude::*;
use jy_derive::JyMenu;

use crate::game::{is_splash, GameStage, GameState, Menu};

pub struct Plugin;

#[derive(Component)]
pub struct SplashScreen;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SplashState {
    None,
    Init,
    Loading,
    End,
}

#[derive(Clone, JyMenu)]
#[allow(dead_code)]
enum MainOption {
    #[i18n(100)]
    Start,
    #[i18n(200)]
    Load,
    #[i18n(300)]
    Exit,
}

#[derive(Clone, JyMenu)]
enum SecondOption {
    Slot0,
    Slot1,
    Slot2,
    Slot3,
}

lazy_static! {
    static ref HASHMAP: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(MainOption::Start.i18n().unwrap(), "重新开始");
        m.insert(MainOption::Load.i18n().unwrap(), "载入进度");
        m.insert(MainOption::Exit.i18n().unwrap(), "离开游戏");
        m
    };
}

#[derive(Component)]
pub struct SplashOption;

#[derive(Component)]
pub struct SplashLoadOption;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Splash).with_system(setup_splash))
            .add_system_set(
                SystemSet::on_exit(GameState::Splash).with_system(despawn_screen::<SplashScreen>),
            )
            .insert_resource(MainOption::Start)
            .add_stage_after(
                CoreStage::Update,
                GameStage::Splash,
                get_splash_stage().with_run_criteria(is_splash),
            )
            .add_state_to_stage(GameStage::Splash, SplashState::None);
    }
}

pub fn get_splash_stage() -> SystemStage {
    let stage = SystemStage::parallel();
    // stage.add_system_set(State::<SplashState>::get_driver());
    info!("init stage");
    stage
        .with_system_set(SystemSet::on_enter(SplashState::Init).with_system(setup_options))
        .with_system_set(
            SystemSet::on_exit(SplashState::Init).with_system(despawn_screen::<SplashOption>),
        )
        .with_system_set(
            SystemSet::on_update(SplashState::Init)
                .with_system(keyboard_menu::<SplashOption, MainOption>)
                .with_system(keyboard_input_system),
        )
        .with_system_set(SystemSet::on_enter(SplashState::Loading).with_system(setup_load_options))
        .with_system_set(
            SystemSet::on_exit(SplashState::Loading)
                .with_system(despawn_screen::<SplashLoadOption>),
        )
        .with_system_set(
            SystemSet::on_update(SplashState::Loading)
                .with_system(keyboard_menu::<SplashLoadOption, SecondOption>)
                .with_system(keyboard_input_sub_system),
        )
}

pub fn setup_splash(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<SplashState>>,
) {
    info!("setup splash");
    let texture_handle = asset_server.load("org/pic/title.png");
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(SplashScreen);
    commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle.clone(),
            ..Default::default()
        })
        .insert(SplashScreen);
    state.set(SplashState::Init).unwrap();
}

/// This system prints 'A' key state
fn keyboard_input_system(
    mut commands: Commands,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<SplashState>>,
    mut game_state: ResMut<State<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
    options: ResMut<MainOption>,
) {
    if keyboard_input.just_released(KeyCode::Return) {
        keyboard_input.clear();
        match *options {
            MainOption::Exit => {
                app_exit_events.send(AppExit);
            }
            MainOption::Start => {
                state.set(SplashState::End).unwrap();
                game_state.set(GameState::Load).unwrap();
            }
            MainOption::Load => {
                commands.insert_resource(SecondOption::Slot0);
                state.set(SplashState::Loading).unwrap();
            }
        }
    }
}

/// This system prints 'A' key state
fn keyboard_menu<Comp, Res>(
    mut commands: Commands,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Text, With<Comp>>,
    options: ResMut<Res>,
) where
    Comp: Component,
    Res: Menu + Sync + Send + 'static,
{
    // on display
    let mut text = query.single_mut();
    text.sections.iter_mut().enumerate().for_each(|(n, v)| {
        if n == options.to_idx() {
            v.style.color = Color::WHITE;
        } else {
            v.style.color = Color::GRAY;
        }
    });

    // onevent
    if keyboard_input.just_pressed(KeyCode::Up) {
        commands.insert_resource(options.up());
        info!("press up");
        keyboard_input.clear();
    }

    if keyboard_input.just_pressed(KeyCode::Down) {
        commands.insert_resource(options.down());
        info!("press down");
        keyboard_input.clear();
    }
}

fn setup_options(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("setup options");
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(SplashOption)
        .with_children(|p| {
            p.spawn_bundle(TextBundle {
                text: Text {
                    // Construct a `Vec` of `TextSection`s
                    sections: (0..MainOption::count())
                        .map(|v| {
                            TextSection {
                                // value: v.to_string(),
                                value: HASHMAP
                                    .get(&MainOption::from(v).i18n().unwrap())
                                    .unwrap()
                                    .to_string()
                                    + "\n",
                                style: TextStyle {
                                    font: asset_server.load("fonts/simsun.ttf"),
                                    font_size: 60.0,
                                    color: if v == 0 { Color::WHITE } else { Color::GRAY },
                                },
                            }
                        })
                        .collect(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(SplashOption);
        });
}

fn setup_load_options(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(SplashLoadOption)
        .with_children(|p| {
            p.spawn_bundle(TextBundle {
                text: Text {
                    // Construct a `Vec` of `TextSection`s
                    sections: (0..SecondOption::count())
                        .map(|v| {
                            TextSection {
                                // value: v.to_string(),
                                value: SecondOption::from(v).to_name() + "\n",
                                style: TextStyle {
                                    font: asset_server.load("fonts/simsun.ttf"),
                                    font_size: 60.0,
                                    color: if v == 0 { Color::WHITE } else { Color::GRAY },
                                },
                            }
                        })
                        .collect(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(SplashLoadOption);
        });
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        debug!("despawn entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

/// This system prints 'A' key state
fn keyboard_input_sub_system(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<SplashState>>,
) {
    if keyboard_input.just_released(KeyCode::Return) {
        keyboard_input.clear();
        state.set(SplashState::Init).unwrap();
    }
}
