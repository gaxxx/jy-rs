#![allow(unused_variables)]

use bevy::app::Events;
use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::game::structs::*;
use crate::game::GameState;
use crate::game::instructs::*;
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
pub struct SpriteMeta(pub Vec<TextureMeta>, pub Timer);

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Interaction)
                .with_system(collect.label("collect"))
                .with_system(dispatch.label("dispatch").after("collect"))
                .with_system(handle_instruct_0.after("dispatch").label("execute"))
                .with_system(handle_instruct_1.after("dispatch").label("execute"))
                .with_system(handle_instruct_2.after("dispatch").label("execute"))
                .with_system(handle_instruct_3.after("dispatch").label("execute"))
                .with_system(handle_instruct_27.after("dispatch").label("execute"))
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
