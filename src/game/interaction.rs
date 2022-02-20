#![allow(unused_variables)]

use bevy::app::Events;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::game::smap::{JyBox, Me, NetCell, Pos};
use crate::game::structs::{Backpack, DData, SData, TextureMeta, Thing, XSCALE, YSCALE};
use crate::game::util::{ImageCache, Status};
use crate::game::GameState;

#[derive(Component)]
pub struct DialogBox;

#[derive(Debug, Clone)]
pub enum JyEvent {
    Dialog(String),
    Cls,
    Sprite,
    Data(i16, i16, Vec<(usize, i16)>),
    Instruct2(i32, i32),
}

#[derive(Component)]
pub struct SpriteMeta(pub Vec<TextureMeta>, pub Timer);

#[derive(Clone, Debug)]
pub struct EventScript {
    wait_input: bool,
    events: Vec<JyEvent>,
    dispatch: Option<JyEvent>,
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Interaction)
                .with_system(collect.label("collect"))
                .with_system(dispatch.label("dispatch").after("collect"))
                .with_system(handle_dialog_event.after("dispatch").label("execute"))
                .with_system(handle_d_data.after("dispatch").label("execute"))
                .with_system(handle_instruct.after("dispatch").label("execute"))
                .with_system(handle_sprite.after("dispatch").label("execute"))
                .with_system(check_input.after("execute")),
        );
    }
}

fn dispatch(
    mut commands: Commands,
    mut mb_ev_script: Option<ResMut<EventScript>>,
    mut state: ResMut<State<GameState>>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let ev_script = mb_ev_script.as_mut().unwrap();
    if ev_script.wait_input {
        return;
    }

    if ev_script.dispatch.is_some() {
        panic!("event should be handled");
    }

    if ev_script.events.len() == 0 {
        commands.remove_resource::<EventScript>();
        state.pop().unwrap();
        return;
    }

    ev_script.dispatch = Some(ev_script.events.remove(0));
}

fn handle_dialog_event(
    mut commands: Commands,
    mut mb_ev_script: Option<ResMut<EventScript>>,
    query: Query<Entity, With<DialogBox>>,
    asset_server: Res<AssetServer>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(JyEvent::Dialog(v)) = ev_script.dispatch.as_ref() {
        commands
            .spawn_bundle(TextBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                text: Text {
                    sections: vec![TextSection {
                        style: TextStyle {
                            font: asset_server.load("fonts/simsun.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                        value: v.to_string(),
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            })
            .insert(DialogBox);
        ev_script.wait_input = true;
        ev_script.dispatch.take();
    } else if let Some(&JyEvent::Cls) = ev_script.dispatch.as_ref() {
        ev_script.dispatch.take();
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn add_to_backpack(backpack: &mut ResMut<Backpack>, thing: i32, size: i32) {
    match backpack
        .items
        .iter_mut()
        .enumerate()
        .find(|(_, (item, _))| *item == thing)
    {
        None => {
            backpack.items.push((thing, size));
        }
        Some((idx, (item, cur))) => {
            *cur += size;
            if *cur == 0 {
                backpack.items.remove(idx);
            }
        }
    }
}

fn handle_instruct(
    d_data: ResMut<DData>,
    things: Res<Vec<Thing>>,
    mut backpack: ResMut<Backpack>,
    mut mb_ev_script: Option<ResMut<EventScript>>,
    sta: Res<Status>,
) {
    if mb_ev_script.is_none() {
        return;
    }
    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(&JyEvent::Instruct2(thing, size)) = ev_script.dispatch.as_ref() {
        add_to_backpack(&mut backpack, thing, size);
        ev_script.dispatch.take();
        let output = format!("得到物品:{} {}", things[thing as usize].name(), size);
        ev_script.events.push(JyEvent::Dialog(output));
        ev_script.events.push(JyEvent::Cls);
    }
}

fn handle_d_data(
    mut commands: Commands,
    mut d_data: ResMut<DData>,
    mut mb_ev_script: Option<ResMut<EventScript>>,
    sta: ResMut<Status>,
    mut meshes: ResMut<Assets<Mesh>>,
    s_data: Res<SData>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut image_cache: ResMut<ImageCache>,
    pos: Res<Pos>,
    query: Query<(&Transform, &JyBox), With<JyBox>>,
) {
    if mb_ev_script.is_none() {
        return;
    }
    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(JyEvent::Data(sc, d, vals)) = ev_script.dispatch.as_ref() {
        let s = if *sc == -2i16 { sta.cur_s } else { *sc as i32 };
        let dv = if *d == -2i16 {
            sta.cur_d.0
        } else {
            *d as usize
        };

        let mut image_update = None;
        for (k, v) in vals {
            d_data.set(s as usize, dv, *k, *v);
            if *k == 7 {
                image_update = Some(*v);
            }
        }

        if image_update.is_some() {
            for (tran, bx) in query.iter() {
                if bx.1 == sta.cur_d.1 && bx.2 == sta.cur_d.2 {
                    let x = bx.1 as f32;
                    let y = bx.2 as f32;
                    commands.entity(bx.0).despawn_recursive();

                    let x_off = -(pos.pos.x - pos.pos.y) as f32 * XSCALE;
                    let y_off = -(-pos.pos.y - pos.pos.x) as f32 * YSCALE;
                    let pic = image_update.unwrap();
                    let mut transform = Transform::from_xyz(
                        (x - y) * XSCALE + x_off,
                        (-x - y) * YSCALE + y_off,
                        3.0,
                    );
                    let (image_h, meta, _) = image_cache.get_image(pic as usize / 2);
                    let d3 =
                        s_data.get_texture(sta.cur_s as usize, x as usize, y as usize, 4) as f32;
                    transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
                    transform.translation.y += meta.3 - meta.1 as f32 / 2. as f32 + d3;
                    let entity = commands
                        .spawn_bundle(MaterialMesh2dBundle {
                            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                            transform: transform.with_scale(Vec3::new(
                                meta.0 as f32,
                                meta.1 as f32,
                                0.,
                            )),
                            material: materials.add(ColorMaterial::from(image_h)),
                            ..Default::default()
                        })
                        .insert(NetCell)
                        .id();
                    commands
                        .entity(entity)
                        .insert(JyBox(entity, x as usize, y as usize));
                }
            }
        }

        ev_script.dispatch.take();
    }
}

fn handle_sprite(
    mut commands: Commands,
    mb_ev_script: Option<ResMut<EventScript>>,
    query: Query<Entity, With<Me>>,
    asset_server: Res<AssetServer>,
    mut sta: ResMut<Status>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let mut ev_script = mb_ev_script.unwrap();
    if let Some(JyEvent::Sprite) = ev_script.dispatch.as_ref() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let x = sta.cur_s_x as f32;
        let y = sta.cur_s_y as f32;
        let x_off = -(sta.cur_s_x - sta.cur_s_y) as f32 * XSCALE;
        let y_off = -(-sta.cur_s_y - sta.cur_s_x) as f32 * YSCALE;
        sta.cur_pic = 2501;
        let mut texture_builder =
            TextureAtlasBuilder::default().initial_size(Vec2::new(XSCALE * 2., YSCALE * 2.));
        let mut metas = vec![];
        (0..28).for_each(|v| {
            if let (image_h, meta, Some(image)) = image_cache.get_image(sta.cur_pic + v) {
                metas.push(meta);
                texture_builder.add_texture(image_h, &image);
            }
        });
        let texture = texture_builder.finish(&mut images).unwrap();
        let texture_atlas = textures.add(texture);
        let mut transform =
            Transform::from_xyz((x - y + 1.) * XSCALE, (-y - x - 1.) * YSCALE + y_off, 3.0);
        let meta = metas[0];

        transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
        // add y scale to make the offset right, I don't know fucking why
        transform.translation.y += YSCALE + meta.3 - meta.1 as f32 / 2.;
        transform.translation.z = 4.;
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas,
                transform,
                ..Default::default()
            })
            .insert(SpriteMeta(metas, Timer::from_seconds(0.5, true)))
            .insert(Me);
        // if let Some((image, meta)) = image_cache.get_image(&*game, game.cur_pic) {
        //     transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
        //     transform.translation.y += meta.3 - meta.1 as f32 / 2.;
        //     commands
        //         .spawn_bundle(SpriteBundle {
        //             transform,
        //             texture: images.add(image),
        //             ..Default::default()
        //         })
        //         .insert(Me);
        // }
        ev_script.dispatch.take();
    }
}

fn collect(
    mut commands: Commands,
    mb_ev_script: Option<ResMut<EventScript>>,
    mut evs: ResMut<Events<JyEvent>>,
) {
    if mb_ev_script.is_some() {
        return;
    }
    if evs.is_empty() {
        return;
    }
    let mut ev_script = EventScript {
        wait_input: false,
        events: vec![],
        dispatch: None,
    };
    evs.drain().for_each(|v| {
        ev_script.events.push(v);
    });

    commands.insert_resource(ev_script);
}

fn check_input(keycode: ResMut<Input<KeyCode>>, mut mb_ev_script: Option<ResMut<EventScript>>) {
    if mb_ev_script.is_some() && mb_ev_script.as_ref().unwrap().wait_input {
        if keycode.just_pressed(KeyCode::Return) || keycode.just_pressed(KeyCode::Space) {
            mb_ev_script.as_mut().unwrap().wait_input = false;
        }
    }
}

pub fn execute_n(
    commands: &mut Commands,
    state: &mut ResMut<State<GameState>>,
    events: &mut ResMut<Events<JyEvent>>,
    event_id: i16,
) {
    let mut enter_inter = true;
    match event_id {
        691 => {
            println!("send dialog");
            events.extend(
                vec![
                    JyEvent::Dialog("Where am I".into()),
                    JyEvent::Cls,
                    JyEvent::Dialog("Who am I".into()),
                    JyEvent::Cls,
                    JyEvent::Sprite,
                ]
                .into_iter(),
            );
        }
        693 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 3500, 3500, 3500, -2, -2, -2,
            ); //  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 174, 100); //  2(2):得到物品[银两][100]
        }
        694 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 3496, 3496, 3496, -2, -2, -2,
            ); //  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 171, 10); //  2(2):得到物品[药材][10]
        }
        695 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 2492, 2492, 2492, -2, -2, -2,
            ); //  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 1, 3); //  2(2):得到物品[精气丸][3]
        }
        696 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 2492, 2492, 2492, -2, -2, -2,
            ); //  --  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 11, 3); //  2(2):得到物品[人蔘][3]
        }
        697 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 2492, 2492, 2492, -2, -2, -2,
            ); //  --  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 3, 3); //   2(2):得到物品[小还丹][3]
        }
        698 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 2492, 2492, 2492, -2, -2, -2,
            ); // --  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 22, 3); // --  2(2):得到物品[黄连解毒丸][3]
        }
        699 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 2612, 2612, 2612, -2, -2, -2,
            ); //   --  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 186, 5); // --  2(2):得到物品[智慧果][5]
        }
        700 => {
            instruct_3(
                events, -2, -2, -2, -2, -1, -1, -1, 3500, 3500, 3500, -2, -2, -2,
            ); //  3(3):修改事件定义:当前场景:当前场景事件编号
            instruct_2(events, 174, 100); //  2(2):得到物品[银两][100]
        }
        i => {
            println!("event i {}", i);
            enter_inter = false;
        }
    }
    if enter_inter {
        state.push(GameState::Interaction).unwrap();
    }
}

fn instruct_2(events: &mut Events<JyEvent>, thing: i32, num: i32) {
    events.send(JyEvent::Instruct2(thing, num));
}

fn instruct_3(
    events: &mut ResMut<Events<JyEvent>>,
    s: i16,
    d: i16,
    v0: i16,
    v1: i16,
    v2: i16,
    v3: i16,
    v4: i16,
    v5: i16,
    v6: i16,
    v7: i16,
    v8: i16,
    v9: i16,
    v10: i16,
) {
    let v = vec![v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10];
    let t = v
        .into_iter()
        .enumerate()
        .fold(vec![], |mut acc, (idx, val)| {
            if val != -2 {
                acc.push((idx, val));
            }
            acc
        });

    let data = JyEvent::Data(s, d, t);
    events.send(data);
}

pub fn setup() {}
