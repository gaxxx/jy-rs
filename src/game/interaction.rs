#![allow(unused_variables)]

use bevy::app::Events;
use bevy::prelude::*;

use crate::game::smap::Me;
use crate::game::structs::{TextureMeta, XSCALE, YSCALE};
use crate::game::util::{ImageCache, Status};
use crate::game::GameState;

#[derive(Component)]
pub struct DialogBox;

#[derive(Debug, Clone)]
pub enum JyEvent {
    Dialog(String),
    Cls,
    Sprite,
    Finish,
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
                .with_system(handle_sprite.after("dispatch").label("execute"))
                .with_system(handle_exit.after("dispatch").label("execute"))
                .with_system(check_input.after("execute")),
        );
    }
}

fn dispatch(mb_ev_script: Option<ResMut<EventScript>>) {
    if mb_ev_script.is_none() {
        return;
    }

    let mut events = mb_ev_script.unwrap();
    if events.wait_input {
        return;
    }

    if events.dispatch.is_some() {
        panic!("event should be handled");
    }

    events.dispatch = Some(events.events.remove(0));
}

fn handle_exit(
    mut commands: Commands,
    mb_ev_script: Option<ResMut<EventScript>>,
    mut state: ResMut<State<GameState>>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let events = mb_ev_script.unwrap();
    if let Some(JyEvent::Finish) = events.dispatch.as_ref() {
        commands.remove_resource::<EventScript>();
        state.pop().unwrap();
    }
}

fn handle_dialog_event(
    mut commands: Commands,
    mb_ev_script: Option<ResMut<EventScript>>,
    query: Query<Entity, With<DialogBox>>,
    asset_server: Res<AssetServer>,
) {
    if mb_ev_script.is_none() {
        return;
    }

    let mut events = mb_ev_script.unwrap();
    if let Some(JyEvent::Dialog(v)) = events.dispatch.as_ref() {
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
        events.wait_input = true;
        events.dispatch.take();
    } else if let Some(&JyEvent::Cls) = events.dispatch.as_ref() {
        events.dispatch.take();
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
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

    let mut events = mb_ev_script.unwrap();
    if let Some(JyEvent::Sprite) = events.dispatch.as_ref() {
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
        transform.translation.y += 10. + meta.3 - meta.1 as f32 / 2.;
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
        events.dispatch.take();
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
    event_id: i32,
) {
    state.push(GameState::Interaction).unwrap();
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
                    JyEvent::Finish,
                ]
                .into_iter(),
            );
        }
        i => {
            println!("event i {}", i);
        }
    }
}

pub fn setup() {}
