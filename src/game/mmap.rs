#![allow(unused_mut)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::ops::ControlFlow;

use bevy::prelude::*;

use crate::game::script::SpriteMeta;
use crate::game::smap::Me;
use crate::game::structs::*;
pub use crate::game::util::ImageCache;
use crate::game::util::{despawn_screen, RenderHelper};
use crate::game::{GameState, GrpAsset};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Mmap).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Mmap)
                    .with_system(movement.label("move"))
                    .with_system(on_event.after("move")),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Mmap).with_system(despawn_screen::<MMapScreen>),
            );
    }
}

#[derive(Component)]
pub struct MMapScreen;

#[allow(unused_mut)]
pub fn setup(
    mut commands: Commands,
    mut _grp_assets: ResMut<Assets<GrpAsset>>,
    _server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut sta: ResMut<SceneStatus>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut images: ResMut<Assets<Image>>,
    mmap_earth: Res<MmapEarth>,
    mmap_surface: Res<MmapSurface>,
    mut render_helper: ResMut<RenderHelper>,
) {
    println!("setup mmap here");
    for h in 0..MMAP_HEIGH {
        for w in 0..MMAP_WIDTH {
            let x = w as f32;
            let y = h as f32;
            let mut transform = sta.offset(x, y, 0.);
            if transform.translation.x.abs() > 600. || transform.translation.y.abs() > 500. {
                continue;
            }
            let offset = h * MMAP_WIDTH + w;
            let pic = mmap_earth.0[offset] / 2;
            if pic > 0 {
                render_helper
                    .render(&mut commands, pic as usize, transform)
                    .map(|v| {
                        commands.entity(v).insert(MMapScreen);
                    });
            }
            let pic = mmap_surface.0[offset] / 2;
            if pic > 0 {
                render_helper
                    .render(&mut commands, pic as usize, transform)
                    .map(|v| {
                        commands.entity(v).insert(MMapScreen);
                    });
            }
        }
    }

    sta.cur_pic = 2501;
    let mut texture_builder =
        TextureAtlasBuilder::default().initial_size(Vec2::new(XSCALE * 2., YSCALE * 2.));
    let mut metas = vec![];
    (0..28).for_each(|v| {
        if let Some((image_h, meta, Some(image))) = image_cache.get_image(sta.cur_pic + v) {
            metas.push(meta);
            texture_builder.add_texture(image_h, &image);
        }
    });
    let texture = texture_builder.finish(&mut images).unwrap();
    let texture_atlas = textures.add(texture);

    let mut transform = sta.offset(1., -1., 3.);
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
        .insert(Me)
        .insert(MMapScreen);
    debug!("start mmap rending");
}

pub fn movement(
    time: Res<Time>,
    mut sta: ResMut<SceneStatus>,
    keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<&mut Transform, (With<MMapScreen>, Without<Me>)>,
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
}

fn on_event() {}
