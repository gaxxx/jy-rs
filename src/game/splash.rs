use std::collections::HashMap;

use lazy_static::lazy_static;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use bevy::{prelude::*};
use bevy::app::AppExit;

use crate::game::{AddSubState, GameState};
use crate::substate;

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

#[derive(Clone, FromPrimitive, ToPrimitive)]
pub enum MainOption {
    Start = 0,
    Load,
    Exit,
}

#[derive(Clone, FromPrimitive, ToPrimitive)]
pub enum SecondOption {
    Slot0 = 10,
    Slot1,
    Slot2,
    Slot3,
}

lazy_static! {
    static ref HASHMAP: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(MainOption::Start as i32, "重新开始");
        m.insert(MainOption::Load as i32, "载入进度");
        m.insert(MainOption::Exit as i32, "离开游戏");
        m.insert(SecondOption::Slot0 as i32, "进度1");
        m.insert(SecondOption::Slot1 as i32, "进度2");
        m.insert(SecondOption::Slot2 as i32, "进度3");
        m.insert(SecondOption::Slot3 as i32, "进度4");
        m
    };
    static ref COUNT: usize = HASHMAP.len();
}

macro_rules! menu_op {
    ($ty : ty, $start : expr, $end : expr) => {
        impl $ty {
            fn up(&self) -> $ty {
                let v = self.to_i32().unwrap();
                if v > $start.to_i32().unwrap() {
                    <$ty>::from_i32(v - 1).unwrap()
                } else {
                    $start
                }
            }

            fn down(&self) -> $ty {
                let v = self.to_i32().unwrap();
                if v < $end.to_i32().unwrap() {
                    <$ty>::from_i32(v + 1).unwrap()
                } else {
                    $end
                }
            }

            fn len() -> usize {
                $end.to_usize().unwrap() - $start.to_usize().unwrap() + 1
            }
        }
    };
}

menu_op!(MainOption, MainOption::Start, MainOption::Exit);
menu_op!(SecondOption, SecondOption::Slot0, SecondOption::Slot3);

#[derive(Component)]
pub struct SplashOption;

#[derive(Component)]
pub struct SplashLoadOption;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(GameState::Splash).with_system(setup_splash))
            .add_system_set(SystemSet::on_exit(GameState::Splash)
                .with_system(despawn_screen::<SplashScreen>)
            )
            .add_sub_state(GameState::Splash, SplashState::None)
            // select options
            .insert_resource(MainOption::Start)

        ;
    }
}

impl AddSubState<GameState, SplashState> for App {
    fn add_sub_state(&mut self, _: GameState, child: SplashState) -> &mut Self {
        self
            .add_state(child)
            .init_non_send_resource::<SubState<SplashState>>()
            .add_system_set(SystemSet::on_update(GameState::Splash)
                                .with_system(splash_update.exclusive_system()),
            )
    }
}

substate!(splash_update, SplashState, {
    let mut stage = SystemStage::parallel();
    stage.add_system_set(State::<SplashState>::get_driver());
    println!("init stage");
    stage
        .with_system_set(SystemSet::on_enter(SplashState::Init).with_system(setup_options))
        .with_system_set(SystemSet::on_exit(SplashState::Init).with_system(despawn_screen::<SplashOption>))
        .with_system_set(
            SystemSet::on_update(SplashState::Init)
                .with_system(keyboard_move_system)
                .with_system(keyboard_input_system)
        )
        .with_system_set(SystemSet::on_enter(SplashState::Loading).with_system(setup_load_options))
        .with_system_set(SystemSet::on_exit(SplashState::Loading).with_system(despawn_screen::<SplashLoadOption>))
        .with_system_set(SystemSet::on_update(SplashState::Loading)
            .with_system(keyboard_move_sub_system)
            .with_system(keyboard_input_sub_system)
        )
});

pub fn setup_splash(mut commands: Commands, asset_server: Res<AssetServer>,
                    mut state: ResMut<State<SplashState>>) {
    println!("setup splash");
    let texture_handle = asset_server.load("pic/title.png");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(SplashScreen);
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        ..Default::default()
    }).insert(SplashScreen);
    state.set(SplashState::Init).unwrap();
}


/// This system prints 'A' key state
fn keyboard_input_system(mut commands: Commands, mut keyboard_input: ResMut<Input<KeyCode>>, mut state: ResMut<State<SplashState>>,
                         mut game_state: ResMut<State<GameState>>,
                         mut app_exit_events: EventWriter<AppExit>, options: ResMut<MainOption>) {
    if keyboard_input.just_released(KeyCode::Return) {
        keyboard_input.clear();
        match *options {
            MainOption::Exit => {
                app_exit_events.send(AppExit);
            }
            MainOption::Start => {
                state.set(SplashState::End).unwrap();
                game_state.set(GameState::Game).unwrap();
            }
            MainOption::Load => {
                commands.insert_resource(SecondOption::Slot0);
                state.set(SplashState::Loading).unwrap();
            }
        }
    }
}

/// This system prints 'A' key state
fn keyboard_move_sub_system(
    mut commands: Commands,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Text, With<SplashLoadOption>>,
    options: ResMut<SecondOption>) {

    // on display
    let mut text = query.single_mut();
    text.sections.iter_mut().enumerate().for_each(|(n, v)| {
        if n + SecondOption::Slot0 as usize == options.clone() as usize {
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

/// This system prints 'A' key state
fn keyboard_move_system(
    mut commands: Commands,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Text, With<SplashOption>>,
    options: ResMut<MainOption>) {

    // on display
    let mut text = query.single_mut();
    text.sections.iter_mut().enumerate().for_each(|(n, v)| {
        if n == options.clone() as usize {
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
    println!("setup options");
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
                    sections: (0..MainOption::len()).map(|v| {
                        TextSection {
                            // value: v.to_string(),
                            value: HASHMAP.get(&(v as i32)).unwrap().to_string() + "\n",
                            style: TextStyle {
                                font: asset_server.load("fonts/simsun.ttf"),
                                font_size: 60.0,
                                color: if v == 0 {
                                    Color::WHITE
                                } else {
                                    Color::GRAY
                                },
                            },
                        }
                    }).collect(),
                    ..Default::default()
                },
                ..Default::default()
            }).insert(SplashOption);
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
                    sections: (0..SecondOption::len()).map(|v| {
                        println!("get map key of {}", v as i32 + SecondOption::Slot0.to_i32().unwrap());
                        TextSection {
                            // value: v.to_string(),
                            value: HASHMAP.get(&(v as i32 + SecondOption::Slot0.to_i32().unwrap())).unwrap().to_string() + "\n",
                            style: TextStyle {
                                font: asset_server.load("fonts/simsun.ttf"),
                                font_size: 60.0,
                                color: if v == 0 {
                                    Color::WHITE
                                } else {
                                    Color::GRAY
                                },
                            },
                        }
                    }).collect(),
                    ..Default::default()
                },
                ..Default::default()
            }).insert(SplashLoadOption);
        });
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    println!("despawn");
    for entity in to_despawn.iter() {
        println!("dispawn entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

/// This system prints 'A' key state
fn keyboard_input_sub_system(mut keyboard_input: ResMut<Input<KeyCode>>, mut state: ResMut<State<SplashState>>) {
    if keyboard_input.just_released(KeyCode::Return) {
        keyboard_input.clear();
        state.set(SplashState::Init).unwrap();
    }
}