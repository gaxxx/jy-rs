use std::fs::File;
use std::io::Read;
use std::mem;

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
fn test_asset() {
    let mut data = vec![];
    let mut idx_data = vec![];
    File::open("./assets/data/smap.grp")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    File::open("./assets/data/smap.idx")
        .unwrap()
        .read_to_end(&mut idx_data)
        .unwrap();

    let mut idx = vec![0];
    let mut cursor = std::io::Cursor::new(idx_data.as_slice());
    while let Ok(ret) = cursor.read_u32::<LittleEndian>() {
        idx.push(ret as usize);
    }
    let _gs = GrpAsset {
        idx,
        data: data.as_slice().into(),
    };
}

#[test]
fn test_structs() {
    assert_eq!(mem::size_of::<Person>(), 202);
}

#[test]
fn load_colors() {
    let mut data = vec![];
    File::open("./assets/data/mmap.col")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let out = structs::load_color(&data);
    assert_eq!(*out.0.get(1).unwrap(), 0xf8f0cc as u32);
}
