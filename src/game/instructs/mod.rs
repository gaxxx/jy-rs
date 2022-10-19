mod instruct_0;
use bevy::prelude::*;
pub use instruct_0::{instruct_0, handle_instruct_0};

mod instruct_1;
pub use instruct_1::{instruct_1, handle_instruct_1};

mod instruct_2;
pub use instruct_2::{instruct_2, handle_instruct_2};

mod instruct_3;
pub use instruct_3::{instruct_3, handle_instruct_3};

mod instruct_27;
pub use instruct_27::{instruct_27, handle_instruct_27};

use crate::game::structs::*;

#[derive(Debug, Clone)]
pub enum JyEvent {
    Dialog(String),
    Cls,
    Sprite,
    Data(i16, i16, Vec<(usize, i16)>),
    Instruct2(i16, i16),
}

#[derive(Clone, Debug)]
pub struct EventScript {
    pub wait_input: bool,
    pub events: Vec<JyEvent>,
    pub dispatch: Option<JyEvent>,
}

#[derive(Component)]
pub struct DialogBox;