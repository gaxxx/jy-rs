#![allow(dead_code)]

use core::default::Default;
use std::mem::size_of;
use std::sync::Arc;

use bevy::{prelude::*};
use bevy::asset::LoadState;
use bevy::utils::HashMap;

use crate::game::{AddSubState, GameState, GrpAsset, GrpLoader, structs};
use crate::game::data_asset::{DataAsset, DataAssetLoader};
use crate::game::structs::{Base, Person, SCENE_HEIGHT, SCENE_WIDTH, SceneInfo, TextureMap, XSCALE, YSCALE};
use crate::substate;

pub struct Plugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum MainState {
    None,
    Load,
    SMap,
    MainMap,
}

fn main_init(mut state: ResMut<State<MainState>>) {
    state.set(MainState::Load).unwrap();
}


impl AddSubState<GameState, MainState> for App {
    fn add_sub_state(&mut self, parent: GameState, child: MainState) -> &mut Self {
        self
            .init_non_send_resource::<SubState<MainState>>()
            .insert_resource(ImageCache::default())
            .add_state(child)
            .add_system_set(
                SystemSet::on_update(parent)
                    .with_system(main_update.exclusive_system())
            )
    }
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<GrpAsset>()
            .add_asset_loader(GrpLoader)
            .add_asset::<DataAsset>()
            .add_asset_loader(DataAssetLoader)
            .add_system_set(
                SystemSet::on_enter(GameState::Game).with_system(main_init)
            )
            .add_sub_state(GameState::Game, MainState::None)
        ;
    }
}

const SIZE: (u32, u32) = (1024, 768);

#[derive(Component)]
pub struct MainScreen;


#[derive(Component)]
pub struct MapScreen;

fn load_data(mut commands: Commands, res: Res<AssetServer>) {
    println!("start to load data");
    let handles = vec![
        res.load("data/ranger.grp"),
        res.load("data/allsin.grp"),
        res.load("data/alldef.grp"),
    ];

    let data_h = vec![
        res.load("data/mmap.col")
    ];

    commands.insert_resource(Game {
        data: None,
        s_all_map: None,
        d_all_map: [].as_ref().into(),
        smap: None,
        cur_s: 0,
        cur_s_x: 0,
        cur_s_y: 0,
        grp_handles: handles,
        data_handles: data_h,
        palette: vec![],
    });
}

struct Thing([u8; 260]);

struct Wugong([u8; 146]);

struct Shop([u8; 30]);

struct GameData {
    base: Base,
    people: Vec<Person>,
    things: Vec<Thing>,
    scenes: Vec<structs::Scene>,
    wukongs: Vec<Wugong>,
    shops: Vec<Shop>,
}

struct Game {
    data: Option<GameData>,
    s_all_map: Option<SceneInfo>,
    d_all_map: Arc<[u8]>,
    smap: Option<TextureMap>,
    cur_s: i32,
    cur_s_x: i32,
    cur_s_y: i32,
    grp_handles: Vec<Handle<GrpAsset>>,
    data_handles: Vec<Handle<DataAsset>>,
    palette: Vec<u32>,
}

impl GameData {
    fn from(asset: GrpAsset) -> Self {
        let gd = GameData {
            base: Base::new(asset.idx(0).unwrap()),
            people: asset.idx(1).unwrap().chunks(size_of::<Person>()).map(|v| {
                Person::new(v)
            }).collect(),
            things: vec![],
            scenes: asset.idx(3).unwrap().chunks(size_of::<structs::Scene>()).map(|v| {
                structs::Scene::new(v)
            }).collect(),
            wukongs: vec![],
            shops: vec![],
        };
        println!("base: {:?}", gd.base);
        println!("talent ones {:?}", gd.people.iter().filter(|v| {
            v.talent > 85
        }).map(|v| {
            (v.name(), v.alias(), v)
        }).collect::<Vec<_>>());
        println!("start scene {:?}", gd.scenes.get(70).map(|v| {
            (v.name(), v)
        }));
        gd
    }
}


fn init_data(mut grp_assets: ResMut<Assets<GrpAsset>>,
             mut ds_assets: ResMut<Assets<DataAsset>>,
             server: Res<AssetServer>,
             mut state: ResMut<State<MainState>>, mut game: ResMut<Game>) {
    println!("start to init data");
    if game.data.is_some() {
        println!("data loaded\n");
        return;
    }

    if game.grp_handles.is_empty() {
        panic!("ooops");
    }

    let iter = game.grp_handles.iter().map(|v| v.id).chain(game.data_handles.iter().map(|v| v.id));


    if server.get_group_load_state(iter) != LoadState::Loaded {
        return;
    }

    game.grp_handles.clone().into_iter().enumerate().for_each(|(idx, handle)| {
        let gs = grp_assets.remove(handle.clone()).unwrap();
        match idx {
            0 => {
                game.data = Some(GameData::from(gs));
            }
            1 => {
                let snum = game.data.as_ref().unwrap().scenes.len();
                let ref_data = gs.data.as_ref();
                println!("snum {}, smap len {} -> to {}", snum, ref_data.len(), snum * structs::SCENE_WIDTH * structs::SCENE_HEIGHT * 12);
                game.s_all_map = Some(SceneInfo::new(gs));
            }
            2 => {
                game.d_all_map = {
                    let snum = game.data.as_ref().unwrap().scenes.len();
                    let ref_data = gs.data.as_ref();
                    println!("dmap {} -> {}", ref_data.len(), snum * structs::DNUM * 11 * 2);
                    ref_data[0..snum * structs::DNUM * 11 * 2].into()
                };
            }
            _ => {}
        }
    });

    if let Some(h) = game.data_handles.clone().into_iter().next() {
        let ds = ds_assets.remove(h.clone()).unwrap();
        game.palette = structs::load_color(ds.data.as_ref());
    }
    game.cur_s = structs::ENTRY_SCENE as i32;
    game.cur_s_x = structs::ENTRY_X;
    game.cur_s_y = structs::ENTRY_Y;
    state.set(MainState::SMap).unwrap();



    /*
    let texture = query.get_single_mut();
    if let Ok(mut img) = texture {
        let mut image = images.get_mut(img.0.clone()).unwrap();
        image.data = [0, 255, 0, 255].iter().map(|v| *v).collect();
        image.resize(Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        })
    }

     */
}

fn update() {}

fn draw(_commands: Commands, _images: ResMut<Assets<Image>>) {}


fn get_main_stage() -> SystemStage {
    let mut stage = SystemStage::parallel();
    stage
        .add_system_set(State::<MainState>::get_driver());
    stage
        .with_system_set(
            SystemSet::on_enter(MainState::Load)
                .with_system(load_data)
        )
        .with_system_set(
            SystemSet::on_update(MainState::Load)
                .with_system(init_data)
        )
        .with_system_set(
            SystemSet::on_enter(MainState::SMap)
                .with_system(load_smap)
        )
        .with_system_set(
            SystemSet::on_update(MainState::SMap)
                .with_system(draw_smap)
        )
        .with_system_set(
            SystemSet::on_enter(MainState::MainMap)
                .with_system(draw)
        ).with_system_set(
        SystemSet::on_update(MainState::MainMap)
            .with_system(update)
    )
}

fn load_smap(res: Res<AssetServer>, mut game: ResMut<Game>) {
    println!("start to load data");
    let handles = vec![
        res.load("data/smap.grp"),
        res.load("data/hdgrp.grp"),
        res.load("data/thing.grp"),
    ];
    game.grp_handles = handles;
}

fn draw_smap(mut commands: Commands,
             mut grp_assets: ResMut<Assets<GrpAsset>>,
             server: Res<AssetServer>,
             mut _state: ResMut<State<MainState>>, mut game: ResMut<Game>,
             mut images: ResMut<Assets<Image>>,
             mut image_cache: ResMut<ImageCache>,
) {
    // load smap & hdgr & thing
    if game.smap.is_none() && !game.grp_handles.is_empty() {
        if server.get_group_load_state(game.grp_handles.iter().map(|v| v.id)) != LoadState::Loaded {
            return;
        }

        game.grp_handles.clone().into_iter().enumerate().for_each(|(idx, handle)| {
            let gs = grp_assets.remove(handle.clone()).unwrap();
            match idx {
                0 => {
                    game.smap = Some(TextureMap::new(gs))
                }
                1 => {
                    // load hrgrp
                }
                2 => {
                    // load thing
                }
                _ => {}
            }
        });

        draw_smap_init(&mut commands, &mut game, &mut images, &server, &mut image_cache);
        return;
    }

    let mut sprites = vec![];
    let mut bundle = commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let _parent = bundle.insert(MainScreen);
    let x_off = 0.0;
    let y_off = 300.0;
    let xf = XSCALE as f32;
    let yf = YSCALE as f32;
    for h in 0..SCENE_HEIGHT {
        for w in 0..SCENE_WIDTH {
            let wf = w as f32;
            let hf = h as f32;
            let earth = game.s_all_map.as_ref().unwrap().get_texture(structs::ENTRY_SCENE, h, w, 0) / 2;
            if earth > 0 {
                let x_scale = (w * structs::XSCALE) as f32;
                let y_scale = (h * structs::YSCALE) as f32;

                let mut transform = Transform::from_xyz(x_scale * 1. - hf * xf + x_off,
                                                        -y_scale * 1. - yf * wf + y_off,
                                                        1.0);

                let mut may_sprite = None;
                if let Some((image_h, xoff, yoff)) = image_cache.get_image(earth) {
                    transform.translation.x += xoff;
                    transform.translation.y += yoff;
                    transform.translation.z += wf + hf;
                    may_sprite = Some(SpriteBundle {
                        texture: image_h.clone(),
                        transform: transform,
                        ..Default::default()
                    });
                } else if let Some((image, xoff, yoff)) = game.smap.as_ref().unwrap().get_image(earth, &game.palette) {
                    transform.translation.x += xoff;
                    transform.translation.y += yoff;
                    transform.translation.z += wf + hf;
                    let handle = images.add(image);
                    image_cache.cached.insert(earth, (handle.clone(), x_off, yoff));
                    may_sprite = Some(SpriteBundle {
                        texture: handle,
                        transform: transform,
                        ..Default::default()
                    });
                }
                if may_sprite.is_some() {
                    sprites.push(may_sprite.unwrap());
                }
            }
        }
    }
    commands.spawn_batch(sprites.into_iter());
}

fn draw_smap_init(commands: &mut Commands, game: &mut ResMut<Game>, images: &mut ResMut<Assets<Image>>,
                  _asset_server: &Res<AssetServer>,
                  image_cache: &mut ResMut<ImageCache>,
) {
    let mut bundle = commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let _parent = bundle.insert(MainScreen);
    let x_off = 0.0;
    let y_off = 300.0;
    let xf = XSCALE as f32;
    let yf = YSCALE as f32;
    let mut sprites = vec![];
    for h in 0..SCENE_HEIGHT {
        for w in 0..SCENE_WIDTH {
            let wf = w as f32;
            let hf = h as f32;
            let earth = game.s_all_map.as_ref().unwrap().get_texture(structs::ENTRY_SCENE, h, w, 0) / 2;
            if earth > 0 {
                let x_scale = (w * structs::XSCALE) as f32;
                let y_scale = (h * structs::YSCALE) as f32;

                let mut transform = Transform::from_xyz(x_scale * 1. - hf * xf + x_off,
                                                        -y_scale * 1. - yf * wf + y_off,
                                                        1.0);

                let mut may_sprite = None;
                if let Some((image_h, xoff, yoff)) = image_cache.get_image(earth) {
                    transform.translation.x += xoff;
                    transform.translation.y += yoff;
                    transform.translation.z += wf + hf;
                    may_sprite = Some(SpriteBundle {
                        texture: image_h.clone(),
                        transform: transform,
                        ..Default::default()
                    });
                } else if let Some((image, xoff, yoff)) = game.smap.as_ref().unwrap().get_image(earth, &game.palette) {
                    transform.translation.x += xoff;
                    transform.translation.y += yoff;
                    transform.translation.z += wf + hf;
                    let handle = images.add(image);
                    image_cache.cached.insert(earth, (handle.clone(), x_off, yoff));
                    may_sprite = Some(SpriteBundle {
                        texture: handle,
                        transform: transform,
                        ..Default::default()
                    });
                }

                if may_sprite.is_some() {
                    sprites.push(may_sprite.unwrap());
                }
            }
        }
    }
    commands.spawn_batch(sprites.into_iter());
    println!("start rending");
}


substate!(main_update, MainState, get_main_stage());



#[derive(Default)]
struct ImageCache {
    cached: HashMap<usize, (Handle<Image>, f32, f32)>,
}

impl ImageCache {
    pub fn get_image(&self, id: usize) -> Option<(Handle<Image>, f32, f32)> {
        if let Some((h, xoff, yoff)) = self.cached.get(&id) {
            Some((h.clone(), *xoff, *yoff))
        } else {
            None
        }
    }
}