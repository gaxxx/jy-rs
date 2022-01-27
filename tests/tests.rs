use std::mem;

use bevy::asset::{FileAssetIo, Handle};
use bevy::prelude::AssetServer;
use bevy::tasks::TaskPool;
use jy::game::{GrpAsset, GrpLoader};
use jy::game::structs::Person;
use jy::prelude::Settings;

#[test]
fn test_settings() {
    let s = Settings::new();
    assert_eq!(s.log_level(), log::Level::Debug);
}

#[test]
fn test_asset() {
    let as_server = AssetServer::new(FileAssetIo::new("./assets"), TaskPool::new());
    as_server.add_loader(GrpLoader);
    let data: Handle<GrpAsset> = as_server.load("./assets/data/smap.grp");
    println!("data is {:?}", data);
    // let ga = as_server.get(data);
}

#[test]
fn test_structs() {
    assert_eq!(mem::size_of::<Person>(), 202);
}