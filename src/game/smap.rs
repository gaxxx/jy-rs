#![allow(unused_mut)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::ops::ControlFlow;

use bevy::app::Events;
use bevy::prelude::*;

use crate::game::script::{JyEvent, SpriteMeta};
use crate::game::structs::*;
pub use crate::game::util::ImageCache;
use crate::game::util::{despawn_screen, RenderHelper};
use crate::game::{script, structs, GameState, GrpAsset};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Smap).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Smap)
                    .with_system(movement.label("move"))
                    .with_system(on_event.after("move")),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Smap).with_system(despawn_screen::<SMapScreen>),
            );
    }
}

#[derive(Component)]
pub struct SMapScreen;

#[allow(unused_mut)]
pub fn setup(
    mut commands: Commands,
    mut _grp_assets: ResMut<Assets<GrpAsset>>,
    _server: Res<AssetServer>,
    mut sta: ResMut<SceneStatus>,
    mut s_data: Res<SData>,
    mut d_data: Res<DData>,
    mut render_helper: ResMut<RenderHelper>,
    mut image_cache: ResMut<ImageCache>,
) {
    println!("setup here");
    for level in 0..=3 {
        for h in 0..SCENE_HEIGHT {
            for w in 0..SCENE_WIDTH {
                let x = w as f32;
                let y = h as f32;

                let id = s_data.get_texture(sta.cur_s, w, h, level);
                if id <= 0 {
                    continue;
                }
                let mut transform = sta.offset(x, y, level as f32);
                let mut pic_id;
                match level {
                    0 => {
                        // earth
                        pic_id = id / 2;
                    }
                    1 => {
                        // building
                        // add building offset
                        transform.translation.y += s_data.get_texture(sta.cur_s, w, h, 4) as f32;
                        pic_id = id / 2;
                    }
                    2 => {
                        // air
                        // add air offset
                        transform.translation.y += s_data.get_texture(sta.cur_s, w, h, 5) as f32;
                        pic_id = id / 2;
                    }
                    3 => {
                        // event
                        // add event offset
                        transform.translation.y += s_data.get_texture(sta.cur_s, w, h, 4) as f32;
                        pic_id = d_data.get_d(sta.cur_s as usize, id as usize, 7) / 2;
                    }
                    _ => {
                        todo!()
                    }
                }
                if pic_id > 0 {
                    render_helper
                        .render(&mut commands, pic_id as usize, transform)
                        .map(|v| {
                            // for event up, we'll update the image after it being triggered.
                            // like when a box is opened, the image would update,
                            // so we need to get the entity and despawn & respawn it with another image.
                            let mut ecmd = commands.entity(v);
                            ecmd.insert(SMapScreen);
                            if level == 3 {
                                ecmd.insert(JyBox(v, w, h));
                            }
                        });
                }
            }
        }
    }

    debug!("start smap rending");

    // draw sprite
    let x = sta.pos.x;
    let y = sta.pos.y;

    if let Some((image_h, meta, _)) = image_cache.get_image(sta.cur_pic) {
        let mut transform = Transform::from_xyz(0., 0., 3.0);
        debug!(
            "sprite init pos {},{}",
            transform.translation.x, transform.translation.y
        );

        let height = s_data.get_texture(sta.cur_s, x as usize, y as usize, 4);
        println!("height is {}", height);

        transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
        transform.translation.y += meta.3 - meta.1 as f32 / 2. + height as f32;
        commands
            .spawn_bundle(SpriteBundle {
                transform,
                texture: image_h,
                ..Default::default()
            })
            .insert(SMapScreen)
            .insert(Me);
    }
}

#[derive(Component)]
pub struct Me;

#[derive(Component)]
pub struct NetCell;

#[derive(Component)]
pub struct JyBox(pub Entity, pub usize, pub usize);

pub fn on_event(
    mut commands: Commands,
    mut events: ResMut<Events<JyEvent>>,
    keyboard_input: ResMut<Input<KeyCode>>,
    d_data: Res<DData>,
    s_data: Res<SData>,
    mut sta: ResMut<SceneStatus>,
    mut state: ResMut<State<GameState>>,
) {
    if sta.is_new_game {
        script::execute_n(&mut commands, &mut state, &mut events, 691);
        sta.is_new_game = false;
        sta.facing = Some(MoveDir::Up);
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Space) && sta.facing.is_some() {
        let next_x = sta.pos.x as i32 + sta.facing.unwrap().pos().0;
        let next_y = sta.pos.y as i32 + sta.facing.unwrap().pos().1;
        let d = s_data.get_texture(sta.cur_s as usize, next_x as usize, next_y as usize, 3);
        if d > 0 {
            sta.cur_d = (d as usize, next_x as usize, next_y as usize);
            let ev = d_data.get_d(sta.cur_s as usize, d as usize, 2);
            if ev > 0 {
                script::execute_n(&mut commands, &mut state, &mut events, ev as i16);
            }
        }
    }
}

pub fn movement(
    time: Res<Time>,
    mut sta: ResMut<SceneStatus>,
    s_data: Res<SData>,
    scenes: Res<Vec<structs::Scene>>,
    d_data: Res<DData>,
    mut state: ResMut<State<GameState>>,
    keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Transform, (With<NetCell>, Without<Me>)>,
    mut me_query: Query<(&mut SpriteMeta, &mut TextureAtlasSprite), With<Me>>,
) {
    let mut moves = HashMap::new();
    moves.insert(KeyCode::Up, MoveDir::Up);
    moves.insert(KeyCode::Down, MoveDir::Down);
    moves.insert(KeyCode::Left, MoveDir::Left);
    moves.insert(KeyCode::Right, MoveDir::Right);

    let mut facing = HashMap::new();
    facing.insert(MoveDir::Up, 0);
    facing.insert(MoveDir::Right, 7);
    facing.insert(MoveDir::Left, 14);
    facing.insert(MoveDir::Down, 21);

    // reset the default pos
    for (mut sprite_meta, mut sprite) in me_query.iter_mut() {
        sprite_meta.1.tick(time.delta());
        if sprite_meta.1.finished() {
            sprite.index = *facing.get(sta.facing.as_ref().unwrap()).unwrap();
        }
    }

    moves.iter().try_for_each(|(code, dir)| {
        if keyboard_input.just_pressed(*code) {
            let last_facing = sta.facing.unwrap();
            sta.facing = MoveDir::from(*code);
            let dir_change = last_facing != sta.facing.unwrap();

            if let Some((mut sprite_meta, mut sprite)) = me_query.iter_mut().next() {
                sprite_meta.1.reset();
                if dir_change {
                    sprite.index = *facing.get(sta.facing.as_ref().unwrap()).unwrap() as usize;
                } else {
                    let base = sprite.index - sprite.index % 7;
                    let next = base + (sprite.index + 1) % 7;
                    sprite.index = next;
                }
            }
            let next_x = sta.pos.x + dir.pos().0 as f32;
            let next_y = sta.pos.y + dir.pos().1 as f32;

            if s_data.get_texture(sta.cur_s as usize, next_x as usize, next_y as usize, 1) > 0 {
                return ControlFlow::Break(());
            }

            let d = s_data.get_texture(sta.cur_s as usize, next_x as usize, next_y as usize, 3);
            if d > 0 {
                if d_data.get_d(sta.cur_s as usize, d as usize, 0) > 0 {
                    return ControlFlow::Break(());
                }
            }

            sta.pos.x = next_x;
            sta.pos.y = next_y;
            for mut iter in query.iter_mut() {
                iter.translation.x += dir.offset().0 * XSCALE;
                iter.translation.y += dir.offset().1 * YSCALE;
            }
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    });

    if (sta.pos.x as i16 == scenes[sta.cur_s].exit_x1
        && sta.pos.y as i16 == scenes[sta.cur_s].exit_y1)
        || (sta.pos.x as i16 == scenes[sta.cur_s].exit_x2
            && sta.pos.y as i16 == scenes[sta.cur_s].exit_y2)
        || (sta.pos.x as i16 == scenes[sta.cur_s].exit_x3
            && sta.pos.y as i16 == scenes[sta.cur_s].exit_y3)
    {
        state.set(GameState::Mmap).unwrap();
    }
}
