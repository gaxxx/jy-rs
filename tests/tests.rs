use std::fs::File;
use std::io::{Read, Result, Write};
use std::mem;

use bevy::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt};

use bevy::log::Level;
use bevy::prelude::{FromWorld, World};
use jy::game::structs::Person;
use jy::game::{structs, GrpAsset};
use jy::prelude::Settings;

#[test]
fn test_settings() {
    let mut world = World::new();
    let s = Settings::from_world(&mut world);
    assert_eq!(s.log_level(), Level::DEBUG);
}

#[test]
fn test_structs() {
    assert_eq!(mem::size_of::<Person>(), 202);
}

#[test]
fn load_colors() {
    let mut data = vec![];
    File::open("./assets/org/data/mmap.col")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let out = structs::load_color(&data);
}


