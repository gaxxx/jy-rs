use std::fmt::Debug;
use std::hash::Hash;

pub use assets::*;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;

mod assets;
mod hint;
mod load;
mod mmap;
pub mod script;
mod smap;
mod sound;
mod splash;
pub mod structs;
pub(crate) mod util;

pub trait Menu {
    fn up(&self) -> Self;
    fn down(&self) -> Self;
    fn to_name(&self) -> String;
    fn to_idx(&self) -> usize;
    fn i18n(&self) -> Option<i32>;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Component)]
pub enum GameState {
    Splash,
    Load,
    Smap,
    Mmap,
    // this should always be pushed with other states
    Interaction,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum GameStage {
    Splash,
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Splash)
            .add_plugin(splash::Plugin)
            .add_plugin(load::Plugin)
            .add_plugin(script::Plugin)
            .add_plugin(hint::Plugin)
            .add_plugin(mmap::Plugin)
            .add_plugin(smap::Plugin);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugin(sound::Plugin);
    }
}

macro_rules! state {
    ($func: ident, $ty : ty, $val : path) => {
        pub fn $func(state: Res<State<$ty>>) -> ShouldRun {
            match *state.current() {
                $val => ShouldRun::Yes,
                _ => ShouldRun::No,
            }
        }
    };
}

state!(is_splash, GameState, GameState::Splash);

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
