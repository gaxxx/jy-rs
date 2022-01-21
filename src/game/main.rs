#![allow(dead_code)]

use std::io::{Cursor, Read};
use std::mem::size_of;
use std::sync::Arc;

use byteorder::{LittleEndian, ReadBytesExt};

use bevy::{prelude::*};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::game::{AddSubState, GameState, GrpAsset, GrpLoader, is_game, structs};
use crate::game::structs::{Base, Person, SCENE_HEIGHT, SCENE_WIDTH, XSCALE, YSCALE};
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

    commands.insert_resource(Game {
        data: None,
        s_all_map: [].as_ref().into(),
        d_all_map: [].as_ref().into(),
        smap: None,
        cur_s: 0,
        cur_s_x: 0,
        cur_s_y: 0,
        handles,
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
    s_all_map: Arc<[u8]>,
    d_all_map: Arc<[u8]>,
    smap: Option<GrpAsset>,
    cur_s: i32,
    cur_s_x: i32,
    cur_s_y: i32,
    handles: Vec<Handle<GrpAsset>>,
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
             mut state: ResMut<State<MainState>>, mut game: ResMut<Game>) {
    println!("start to init data");
    if game.data.is_some() {
        println!("data loaded\n");
        return;
    }
    for i in game.handles.iter() {
        if grp_assets.get(i.clone()).is_none() {
            return;
        }
    }

    game.handles.clone().into_iter().enumerate().for_each(|(idx, handle)| {
        let gs = grp_assets.remove(handle.clone()).unwrap();
        match idx {
            0 => {
                game.data = Some(GameData::from(gs));
            }
            1 => {
                game.s_all_map = {
                    let snum = game.data.as_ref().unwrap().scenes.len();
                    let ref_data = gs.data.as_ref();
                    println!("smap len {} -> to {}", ref_data.len(), snum * structs::SCENE_WIDTH * structs::SCENE_HEIGHT * 12);
                    ref_data[0..snum * structs::SCENE_WIDTH * structs::SCENE_HEIGHT * 12].into()
                };
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
    game.cur_s = structs::ENTRY_SCENE;
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

fn draw(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    println!("draw data");
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 255, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn_bundle(ImageBundle {
        node: Node { size: Vec2::new(SIZE.0 as f32, SIZE.1 as f32) },
        image: UiImage::from(image.clone()),
        /*
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..Default::default()
        },
        texture: image.clone(),
         */
        ..Default::default()
    })
        .insert(MapScreen)
        .insert(MainScreen);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(MainScreen);
}


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

fn load_smap(mut _commands: Commands, res: Res<AssetServer>, mut game: ResMut<Game>) {
    println!("start to load data");
    let handles = vec![
        res.load("data/smap.grp"),
        res.load("data/hdgrp.grp"),
        res.load("data/thing.grp"),
    ];
    game.handles = handles;
}

fn draw_smap(mut commands: Commands,
             mut grp_assets: ResMut<Assets<GrpAsset>>,
             mut _state: ResMut<State<MainState>>, mut game: ResMut<Game>,
             mut images: ResMut<Assets<Image>>) {
    // load smap & hdgr & thing
    if game.smap.is_none() {
        for i in game.handles.iter() {
            if grp_assets.get(i.clone()).is_none() {
                return;
            }
        }

        game.handles.clone().into_iter().enumerate().for_each(|(idx, handle)| {
            let gs = grp_assets.remove(handle.clone()).unwrap();
            println!("test data");
            match idx {
                0 => {
                    game.smap = Some(gs)
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

        draw_smap_init(&mut commands, &mut game, &mut images);
    }
}

fn draw_smap_init(commands: &mut Commands, game: &mut ResMut<Game>, images: &mut ResMut<Assets<Image>>) {
    println!("start rending");
    game.smap.as_ref().map(|v| {
        let x = structs::ENTRY_X;
        let y = structs::ENTRY_Y;
        let xoff = -1;
        let yoff = -1;
        //根据g_Surface的剪裁来确定循环参数。提高绘制速度
        let istart = (0 - 1024 / 2) / (2 * XSCALE as i32) - 1 - 2;
        let iend = (1024 - 1024 / 2) / (2 * XSCALE as i32) + 1 + 2;

        let jstart = (0 - 768 / 2) / (2 * YSCALE as i32) - 1;
        let jend = (768 - 768 / 2) / (2 * YSCALE as i32) + 1;
        for j in 0..=2 * (jend - jstart) + 16 {
            for i in istart..=iend {
                let i1 = i + j / 2 + jstart;
                let j1 = -i + j / 2 + j % 2 + jstart;
                println!("i1:{}, j1:{}\n", i1, j1);


                let x1 = XSCALE as i32 * (i1 - j1) + 1024 as i32 / 2;
                let y1 = YSCALE as i32 * (i1 + j1) + 768 as i32 / 2;

                println!("x1:{}, y1:{}", x1, y1);
                let xx = x + i1 + xoff;
                let yy = y + j1 + yoff;
                println!("xx:{}, yy:{}", xx, yy);

                if (xx >= 0) && (xx < SCENE_WIDTH as i32) && (yy >= 0) && (yy < SCENE_HEIGHT as i32) {
                    let start = structs::SCENE_WIDTH * structs::SCENE_HEIGHT * structs::ENTRY_SCENE as usize * 6 as usize
                        + yy as usize * structs::SCENE_WIDTH + xx as usize;
                    let d0 = &game.s_all_map.as_ref()[start..start + 2];
                    let d0 = i16::from_be_bytes(d0.try_into().unwrap());
                    println!("d0:{}", d0);
                    if d0 > 0 {
                        let data = v.idx(4).unwrap();
                        let mut c = Cursor::new(data);
                        let w = c.read_i16::<LittleEndian>().unwrap();
                        let h = c.read_i16::<LittleEndian>().unwrap();
                        let xoff = c.read_i16::<LittleEndian>().unwrap();
                        let yoff = c.read_i16::<LittleEndian>().unwrap();
                        println!("data len {} w:{}, h:{}, xoff:{}, yoff:{}",
                                 data.len(),
                                 w, h, xoff, yoff
                        );
                        drop(c);

                        commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(MainScreen);
                        let mut image = Image::new_fill(
                            Extent3d {
                                width: SIZE.0,
                                height: SIZE.1,
                                depth_or_array_layers: 1,
                            },
                            TextureDimension::D2,
                            &data[8..],
                            TextureFormat::Rgba8Unorm,
                        );
                        image.texture_descriptor.usage =
                            TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
                        let image = images.add(image);

                        commands.spawn_bundle(ImageBundle {
                            node: Node { size: Vec2::new(SIZE.0 as f32, SIZE.1 as f32) },
                            image: UiImage::from(image.clone()),
                            ..Default::default()
                        })
                            .insert(MapScreen)
                            .insert(MainScreen);

                        return;
                    }
                }
            }
        }
    });
}


substate!(main_update, MainState, get_main_stage());
