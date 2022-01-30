use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::{prelude::*};
use bevy::ecs::schedule::{ShouldRun, StateData};
pub use data_asset::DataAssetLoader;
pub use grp_asset::{GrpAsset, GrpLoader};

mod splash;
mod sound;
mod main;
mod grp_asset;
pub mod structs;
mod data_asset;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Component)]
pub enum GameState {
    Splash,
    Game,
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(GameState::Splash)
            .add_plugin(splash::Plugin)
            .add_plugin(sound::Plugin)
            .add_plugin(main::Plugin)
            .add_system_set(
                SystemSet::on_update(GameState::Game)
            );
    }
}

macro_rules! state {
    ($func: ident, $ty : ty, $val : path) => {
        pub fn $func(state : Res<State<$ty>>) -> ShouldRun {
            match *state.current() {
                $val => {
                    ShouldRun::Yes
                }
                _ => {
                    ShouldRun::No
                }
            }
        }
    };
}

state!(is_game, GameState, GameState::Game);
state!(is_splash, GameState, GameState::Splash);

/// Used internally to play audio on the current "audio device"
pub struct SubState<T>
    where T: StateData
{
    val: Option<SystemStage>,
    phantom: PhantomData<T>,
}

impl<T> Default for SubState<T> where T: StateData {
    fn default() -> Self {
        Self {
            val: None,
            phantom: PhantomData,
        }
    }
}

trait AddSubState<P: StateData, C: StateData> {
    fn add_sub_state(&mut self, parent: P, child: C) -> &mut Self;
}

#[macro_export]
macro_rules! substate {
    ($func: ident, $ty : ty, $val : expr) => {
        use crate::game::SubState;
        #[allow(unused_mut)]
        pub fn $func(world: &mut World) {
            let mut maybe_substate = world.cell().get_non_send_mut::<SubState<$ty>>().unwrap().val.take();
    if maybe_substate.is_none() {
        println!("steup sub stage {}", stringify!($ty));
        let mut stage = $val;
        maybe_substate = Some(stage);
    }
    if let Some(stage) = maybe_substate.as_mut() {
        stage.run(world);
    }
    world.cell().get_non_send_mut::<SubState<$ty>>().unwrap().val = maybe_substate;
            }
        }
}

#[macro_export]
macro_rules! read {
    ($ident : ident, u8) => {
        $ident.read_u8().unwrap()
    };
    ($ident : ident, i16) => {
        $ident.read_i16::<LittleEndian>().unwrap()
    };
    ($ident : ident, u16) => {
        $ident.read_u16::<LittleEndian>().unwrap()
    };
    ($ident : ident, $val : ident, u8) => {
        $ident.read($vall);
    };
}

