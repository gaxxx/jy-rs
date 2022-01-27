#![allow(dead_code)]

use std::mem::size_of;
use std::sync::Arc;

use bevy::{prelude::*};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::game::{AddSubState, GameState, GrpAsset, GrpLoader, structs};
use crate::game::structs::{Base, Person};
use crate::substate;

pub struct Plugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum MainState {
    None,
    Load,
    Move,
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

const SIZE: (u32, u32) = (1280, 720);

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

    commands.insert_resource(Game { data: None, smap: [].as_ref().into(), dmap: [].as_ref().into(), handles });
}

struct GameOfLifeImage(Handle<Image>);


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
    smap: Arc<[u8]>,
    dmap: Arc<[u8]>,
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
                game.smap = {
                    let snum = game.data.as_ref().unwrap().scenes.len();
                    let ref_data = gs.data.as_ref();
                    ref_data[0..snum * structs::SCENE_WIDTH * structs::SCENE_HEIGHT * 12].into()
                };
            }
            2 => {
                game.dmap = {
                    let snum = game.data.as_ref().unwrap().scenes.len();
                    let ref_data = gs.data.as_ref();
                    ref_data[0..snum * structs::DNUM * 11 * 2].into()
                };
            }
            _ => {}
        }
    });
    state.set(MainState::Move).unwrap();



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
    commands.insert_resource(GameOfLifeImage(image))
}

substate!(main_update, MainState, {
    let mut stage = SystemStage::parallel();
        stage.add_system_set(State::<MainState>::get_driver());
        stage.with_system_set(
            SystemSet::on_enter(MainState::Load)
                .with_system(load_data)
        ).with_system_set(
            SystemSet::on_update(MainState::Load)
                .with_system(init_data)
        ).with_system_set(
            SystemSet::on_enter(MainState::Move)
                .with_system(draw)
        ).with_system_set(
            SystemSet::on_update(MainState::Move)
                .with_system(update)
        )

});




