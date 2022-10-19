#![allow(unused_variables)]

use bevy::app::Events;
use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::game::smap::{JyBox, Me, SMapScreen};
use crate::game::structs::*;
use crate::game::util::{ImageCache, RenderHelper};
use crate::game::GameState;
use rlua::Lua;
use std::fs::File;
use std::io::*;
use std::sync::Mutex;

#[cfg(test)]
pub mod test {
    use rlua::Lua;

    #[test]
    fn test_lua() {
        let lua = Lua::new();

        // In order to interact with Lua values at all, you must do so inside a callback given to the
        // `Lua::context` method.  This provides some extra safety and allows the rlua API to avoid some
        // extra runtime checks.
        lua.context(|lua_ctx| {
            // You can get and set global variables.  Notice that the globals table here is a permanent
            // reference to _G, and it is mutated behind the scenes as Lua code is loaded.  This API is
            // based heavily around sharing and internal mutation (just like Lua itself).

            let globals = lua_ctx.globals();
            let print_str = lua_ctx
                .create_function(|_, s: String| {
                    println!("{}", s);
                    Ok(true)
                })
                .unwrap();
            globals.set("print_str", print_str).unwrap();

            assert_eq!(
                lua_ctx
                    .load(
                        r#"
                            print_str("huhuhu")
                            "#,
                    )
                    .eval::<bool>()
                    .unwrap(),
                true
            );
        });
    }
}

#[derive(Component)]
pub struct DialogBox;

#[derive(Debug, Clone)]
pub enum JyEvent {
    Dialog(String),
    Cls,
    Sprite,
    Data(i16, i16, Vec<(usize, i16)>),
    Instruct2(i16, i16),
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

fn add_to_backpack(backpack: &mut ResMut<Backpack>, thing: i16, size: i16) {
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
    sta: Res<SceneStatus>,
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
    sta: ResMut<SceneStatus>,
    mut render_helper: ResMut<RenderHelper>,
    s_data: Res<SData>,
    query: Query<(&Transform, &JyBox), With<JyBox>>,
) {
    if mb_ev_script.is_none() {
        return;
    }
    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(JyEvent::Data(sc, d, vals)) = ev_script.dispatch.as_ref() {
        let s = if *sc == -2i16 {
            sta.cur_s
        } else {
            *sc as usize
        };
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
                    let mut trans = Transform::from_translation(sta.pos.to_real(x, y, 3.));
                    trans.translation.y +=
                        s_data.get_texture(sta.cur_s, x as usize, y as usize, 4) as f32;
                    render_helper
                        .render(
                            &mut commands,
                            MapType::Smap,
                            image_update.clone().unwrap() as usize / 2,
                            trans,
                        )
                        .map(|v| {
                            commands.entity(v).insert(SMapScreen);
                        });
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
    sta: ResMut<SceneStatus>,
    image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
    mut render_helper: ResMut<RenderHelper>,
    textures: ResMut<Assets<TextureAtlas>>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let mut ev_script = mb_ev_script.unwrap();
    if let Some(JyEvent::Sprite) = ev_script.dispatch.as_ref() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let entity = render_helper.render_sprite(&mut commands, MapType::Smap, &mut images);
        commands.entity(entity).insert(Me).insert(SMapScreen);
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

lazy_static! {
    pub static ref S_LUA: Mutex<Lua> = Mutex::new(init_lua());
    pub static ref S_EVENT_QUE: Mutex<Vec<JyEvent>> = Mutex::new(vec![]);
}

pub fn execute_n(
    commands: &mut Commands,
    state: &mut ResMut<State<GameState>>,
    events: &mut ResMut<Events<JyEvent>>,
    event_id: i16,
) {
    let mut ev_guard = S_EVENT_QUE.lock().unwrap();
    ev_guard.clear();
    drop(ev_guard);
    println!("exec event {}", event_id);
    S_LUA.lock().unwrap().context(|lua_ctx| {
        let mut data = vec![];
        File::open(format!("assets/script/oldevent_{}.lua", event_id))
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        lua_ctx.load(data.as_slice()).exec().unwrap();
        let mut ev_guard = S_EVENT_QUE.lock().unwrap();
        events.extend(ev_guard.clone().into_iter());
        ev_guard.clear();
    });

    state.push(GameState::Interaction).unwrap();
}

fn instruct_27(id: i16, start_pic: i16, end_pic: i16) -> JyEvent {
    JyEvent::Sprite
}

fn instruct_2(thing: i16, num: i16) -> JyEvent {
    JyEvent::Instruct2(thing, num)
}

fn instruct_1(talk_id: i32, head_id: i32, flag: i32) -> JyEvent {
    JyEvent::Dialog("Hello world".into())
}

fn instruct_0() -> JyEvent {
    JyEvent::Cls
}

fn instruct_3(
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
) -> JyEvent {
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

    JyEvent::Data(s, d, t)
}

fn init_lua() -> Lua {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals
            .set(
                "instruct_0",
                lua_ctx
                    .create_function_mut(|_, ()| {
                        let ev = instruct_0();
                        let mut ev_guard = S_EVENT_QUE.lock().unwrap();
                        ev_guard.push(ev);
                        Ok(true)
                    })
                    .unwrap(),
            )
            .unwrap();

        globals
            .set(
                "instruct_1",
                lua_ctx
                    .create_function_mut(|_, (talk_id, head_id, flag): (i32, i32, i32)| {
                        let ev = instruct_1(talk_id, head_id, flag);
                        let mut ev_guard = S_EVENT_QUE.lock().unwrap();
                        ev_guard.push(ev);
                        Ok(true)
                    })
                    .unwrap(),
            )
            .unwrap();

        globals
            .set(
                "instruct_2",
                lua_ctx
                    .create_function_mut(|_, (thing_id, num): (i16, i16)| {
                        let ev = instruct_2(thing_id, num);
                        let mut ev_guard = S_EVENT_QUE.lock().unwrap();
                        ev_guard.push(ev);
                        Ok(true)
                    })
                    .unwrap(),
            )
            .unwrap();

        globals
            .set(
                "instruct_3",
                lua_ctx
                    .create_function_mut(
                        |_,
                         (s, d, v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10): (
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                            i16,
                        )| {
                            let ev = instruct_3(s, d, v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10);
                            let mut ev_guard = S_EVENT_QUE.lock().unwrap();
                            ev_guard.push(ev);
                            Ok(true)
                        },
                    )
                    .unwrap(),
            )
            .unwrap();

        globals
            .set(
                "instruct_27",
                lua_ctx
                    .create_function_mut(|_, (id, start_pic, end_pic): (i16, i16, i16)| {
                        let ev = instruct_27(id, start_pic, end_pic);
                        let mut ev_guard = S_EVENT_QUE.lock().unwrap();
                        ev_guard.push(ev);
                        Ok(true)
                    })
                    .unwrap(),
            )
            .unwrap();
    });
    lua
}
