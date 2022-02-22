use std::mem::size_of;

use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::game::assets::*;
use crate::game::script::JyEvent;
use crate::game::structs::*;
use crate::game::util::{ImageCache, RenderHelper};
use crate::game::{structs, GameState};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GrpAsset>()
            .add_event::<JyEvent>()
            .add_asset_loader(GrpLoader)
            .add_asset::<DataAsset>()
            .add_asset_loader(DataAssetLoader)
            .add_system_set(SystemSet::on_enter(GameState::Load).with_system(loading))
            .add_system_set(SystemSet::on_update(GameState::Load).with_system(load));
    }
}

pub struct GameLoad {
    pub grp_handles: Vec<Handle<GrpAsset>>,
    pub data_handles: Vec<Handle<DataAsset>>,
}

pub fn load(
    mut commands: Commands,
    mut grp_assets: ResMut<Assets<GrpAsset>>,
    mut ds_assets: ResMut<Assets<DataAsset>>,
    game_load: Res<GameLoad>,
    server: Res<AssetServer>,
    mut state: ResMut<State<GameState>>,
) {
    let iter = game_load
        .grp_handles
        .iter()
        .map(|v| v.id)
        .chain(game_load.data_handles.iter().map(|v| v.id));

    if server.get_group_load_state(iter) != LoadState::Loaded {
        return;
    }
    println!("load {}", game_load.grp_handles.len());

    let mut scene_num = 0;
    game_load
        .grp_handles
        .clone()
        .into_iter()
        .enumerate()
        .for_each(|(idx, handle)| {
            let gs = grp_assets.remove(handle.clone()).unwrap();
            match idx {
                0 => {
                    let gd = GameData::new(gs);

                    scene_num = gd.scenes.len();
                    commands.insert_resource(gd.base);
                    commands.insert_resource(gd.scenes);
                    commands.insert_resource(gd.people);
                    commands.insert_resource(gd.shops);
                    commands.insert_resource(gd.things);
                    commands.insert_resource(gd.wukongs);
                    commands.insert_resource(Backpack::default())
                }
                1 => {
                    debug!(
                        "snum {}, smap len {} -> to {}",
                        scene_num,
                        gs.data.len(),
                        scene_num * structs::SCENE_WIDTH * structs::SCENE_HEIGHT * 12
                    );
                    commands.insert_resource(SData::new(gs));
                }
                2 => {
                    debug!(
                        "dmap {} -> {}",
                        gs.data.len(),
                        scene_num * structs::DNUM * 11 * 2
                    );
                    commands.insert_resource(DData::new(gs));
                }
                3 => {
                    commands.insert_resource(TextureMap::new(gs));
                }
                _ => {}
            }
        });

    game_load
        .data_handles
        .clone()
        .into_iter()
        .enumerate()
        .for_each(|(idx, handle)| {
            let ds = ds_assets.remove(handle.clone()).unwrap();
            match idx {
                0 => {
                    // mmap.col
                    // color
                    commands.insert_resource(structs::load_color(&ds.data));
                }
                1 => {
                    // earth.002
                    commands.insert_resource(MmapEarth(structs::load_i16(&ds.data)));
                }
                2 => {
                    // surface.002
                    commands.insert_resource(MmapSurface(structs::load_i16(&ds.data)));
                }
                3 => {
                    // building
                    commands.insert_resource(MmapBuilding(structs::load_i16(&ds.data)));
                }
                4 => {
                    // buildx
                    commands.insert_resource(MmapBuildX(structs::load_i16(&ds.data)));
                }
                5 => {
                    // buildy
                    commands.insert_resource(MmapBuildY(structs::load_i16(&ds.data)));
                }
                _ => {}
            }
        });
    commands.init_resource::<ImageCache>();

    let mut sta = SceneStatus::default();
    sta.cur_s = structs::ENTRY_SCENE;
    sta.pos.x = structs::ENTRY_X as f32;
    sta.pos.y = structs::ENTRY_Y as f32;
    sta.cur_pic = NEW_PERSON;
    sta.is_new_game = true;
    if sta.is_new_game {
        state.set(GameState::Smap).unwrap();
    }
    commands.insert_resource(sta);
    commands.init_resource::<RenderHelper>();
    commands.remove_resource::<GameLoad>();
}

pub fn loading(mut commands: Commands, res: Res<AssetServer>) {
    debug!("start to load data");
    let handles = vec![
        res.load("org/data/ranger.grp"),
        res.load("org/data/allsin.grp"),
        res.load("org/data/alldef.grp"),
        // smap
        res.load("org/data/smap.grp"),
        res.load("org/data/hdgrp.grp"),
        res.load("org/data/thing.grp"),
    ];

    let data_h = vec![
        res.load("org/data/mmap.col"),
        // mmap
        res.load("org/data/earth.002"),
        res.load("org/data/surface.002"),
        res.load("org/data/building.002"),
        res.load("org/data/buildx.002"),
        res.load("org/data/buildy.002"),
    ];

    commands.insert_resource(GameLoad {
        grp_handles: handles,
        data_handles: data_h,
    });
}

struct GameData {
    pub base: Base,
    pub people: Vec<Person>,
    pub things: Vec<Thing>,
    pub scenes: Vec<structs::Scene>,
    pub wukongs: Vec<Wugong>,
    pub shops: Vec<Shop>,
}

impl GameData {
    pub fn new(asset: GrpAsset) -> Self {
        let gd = GameData {
            base: Base::new(asset.idx(0).unwrap()),
            people: asset
                .idx(1)
                .unwrap()
                .chunks(size_of::<Person>())
                .map(|v| Person::new(v))
                .collect(),
            things: asset
                .idx(2)
                .unwrap()
                .chunks(size_of::<Thing>())
                .map(|v| Thing::new(v))
                .collect(),
            scenes: asset
                .idx(3)
                .unwrap()
                .chunks(size_of::<structs::Scene>())
                .map(|v| structs::Scene::new(v))
                .collect(),
            wukongs: vec![],
            shops: vec![],
        };
        debug!("base: {:?}", gd.base);
        debug!(
            "talent ones {:?}",
            gd.people
                .iter()
                .filter(|v| { v.talent > 85 })
                .map(|v| { (v.name(), v.alias(), v) })
                .collect::<Vec<_>>()
        );
        debug!(
            "start scene {:?}",
            gd.scenes.get(70).map(|v| { (v.name(), v) })
        );
        gd
    }
}
