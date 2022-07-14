#![allow(unused_mut)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use bevy::prelude::*;

use crate::game::script::SpriteMeta;
use crate::game::smap::Me;
use crate::game::structs::*;
pub use crate::game::util::ImageCache;
use crate::game::util::{despawn_screen, Canvas, PosXY, RenderHelper};
use crate::game::{GameState, GrpAsset};

pub struct Plugin;

const X_CACHE: f32 = 700.;
const Y_CACHE: f32 = 600.;

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

#[derive(Component, PartialEq, Eq, Hash)]
pub struct MMapLocation(pub usize, pub usize);

#[derive(Component)]
pub struct MMapScreen;

#[derive(Default)]
pub struct MMapStatus {
    pub cur_pic: usize,
    pub pos: PosXY,
}

struct MMapCanvasWriter<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    location_set: &'a mut HashSet<MMapLocation>,
    render_helper: &'a mut RenderHelper,
    mmap_earth: &'a MmapEarth,
    mmap_surface: &'a MmapSurface,
    mmap_buiding: &'a MmapBuilding,
    mmap_buildx : &'a MmapBuildX,
    mmap_buildy : &'a MmapBuildY,
}

impl<'a, 'w, 's> MMapCanvasWriter<'a, 'w, 's> {
    pub fn new(
        commands: &'a mut Commands<'w, 's>,
        location_set: &'a mut HashSet<MMapLocation>,
        render_helper: &'a mut RenderHelper,
        mmap_earth: &'a MmapEarth,
        mmap_surface: &'a MmapSurface,
        mmap_buiding: &'a MmapBuilding,
        mmap_buildx : &'a MmapBuildX,
        mmap_buildy : &'a MmapBuildY,
    ) -> MMapCanvasWriter<'a, 'w, 's> {
        Self {
            commands,
            location_set,
            render_helper,
            mmap_earth,
            mmap_surface,
            mmap_buiding,
            mmap_buildx,
            mmap_buildy
        }
    }

    fn draw_pic(&mut self, pic : i16, w : usize, h : usize,  mut p: Vec3) {
        self.render_helper
                    .render(self.commands, MapType::Mmap,pic as usize, Transform::from_translation(p))
                    .map(|v| {
                        self.commands
                            .entity(v)
                            .insert(MMapLocation(w, h))
                            .insert(MMapScreen);
                    });
    }

    fn draw_at(&mut self, x: i32, y: i32, mut p: Vec3) {
        if x >= 0 && x <= MMAP_WIDTH as i32 && y >= 0 && y < MMAP_HEIGH as i32 {
            let w = x as usize;
            let h = y as usize;
            let loc = MMapLocation(w, h);
            if self.location_set.contains(&loc) {
                return;
            }
            self.location_set.insert(MMapLocation(w, h));

            let offset = h * MMAP_WIDTH + w;
            let mut pic = self.mmap_earth.0[offset] / 2;
            // println!("draw at x:{}, y:{}", x, y);
            if pic > 0 {
                println!("draw earth {}", pic);
                self.draw_pic(pic, w, h, p);
                
            }

            pic = self.mmap_surface.0[offset] /2 ;

            p.z += 1.0;
            if pic > 0 {
                // println!("draw surfce {}", pic);
                self.draw_pic(pic, w, h, p);
            }

            pic = self.mmap_buiding.0[offset] / 2;
            p.z += 1.0;
            if pic > 0 {
                println!("draw buiding {} at x:y {}:{} ", pic, x, y);
                self.draw_pic(pic, w, h, p);
            }
        }
    }
}

#[allow(unused_mut)]
pub fn setup(
    mut commands: Commands,
    mut _grp_assets: ResMut<Assets<GrpAsset>>,
    _server: Res<AssetServer>,
    mut sta: ResMut<MMapStatus>,
    mut images: ResMut<Assets<Image>>,
    mmap_earth: Res<MmapEarth>,
    mmap_surface: Res<MmapSurface>,
    mmap_buiding: Res<MmapBuilding>,
    mmap_buildx: Res<MmapBuildX>,
    mmap_buildy: Res<MmapBuildY>,
    mut render_helper: ResMut<RenderHelper>,
) {
    let mut location_set = HashSet::new();
    println!("setup mmap here from {}:{}", sta.pos.x, sta.pos.y);

    let mut m = MMapCanvasWriter::new(
        &mut commands,
        &mut location_set,
        &mut render_helper,
        &mmap_earth,
        &mmap_surface,
        &mmap_buiding,
        &mmap_buildx,
        &mmap_buildy
    );
    Canvas::update(&sta.pos, |x: i32, y: i32, p: Vec3| {
        m.draw_at(x, y, p);
    });

    commands.insert_resource(location_set);

    let entity = render_helper.render_sprite(&mut commands, MapType::Mmap, &mut images);
    commands.entity(entity).insert(Me).insert(MMapScreen);
    debug!("start mmap rending");
}

pub fn movement(
    mut commands: Commands,
    time: Res<Time>,
    mut mta: ResMut<MMapStatus>,
    mut location_set: ResMut<HashSet<MMapLocation>>,
    keyboard_input: ResMut<Input<KeyCode>>,
    mmap_earth: Res<MmapEarth>,
    mmap_surface: Res<MmapSurface>,
    mmap_buiding: Res<MmapBuilding>,
    mmap_buildx: Res<MmapBuildX>,
    mmap_buildy: Res<MmapBuildY>,
    mut render_helper: ResMut<RenderHelper>,
    mut query: Query<(&mut Transform, &mut MMapLocation, Entity), (With<MMapScreen>, Without<Me>)>,
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
            sprite.index = *facing.get(&mta.pos.facing()).unwrap();
        }
    }

    moves.iter().try_for_each(|(code, dir)| {
        if keyboard_input.pressed(*code) {
            let last_facing = mta.pos.facing.unwrap();
            mta.pos.facing = MoveDir::from(*code);
            let dir_change = last_facing != mta.pos.facing.unwrap();

            if let Some((mut sprite_meta, mut sprite)) = me_query.iter_mut().next() {
                sprite_meta.1.reset();
                if dir_change {
                    sprite.index = *facing.get(&mta.pos.facing()).unwrap() as usize;
                } else {
                    let base = sprite.index - sprite.index % 7;
                    let next = base + (sprite.index + 1) % 7;
                    sprite.index = next;
                }
            }
            let next_x = mta.pos.x + dir.pos().0 as f32;
            let next_y = mta.pos.y + dir.pos().1 as f32;
            // update mov of others
            for (mut tt, loc, entity) in query.iter_mut() {
                tt.translation.x += dir.offset().0 * XSCALE;
                tt.translation.y += dir.offset().1 * YSCALE;
                if tt.translation.x.abs() > X_CACHE || tt.translation.y.abs() > Y_CACHE {
                    commands.entity(entity).despawn_recursive();
                    location_set.remove(&loc);
                }
            }

            mta.pos.update(next_x as usize, next_y as usize);
            {
                let mut m = MMapCanvasWriter::new(
                    &mut commands,
                    &mut location_set,
                    &mut render_helper,
                    &mmap_earth,
                    &mmap_surface,
                    &mmap_buiding,
                    &mmap_buildx,
                    &mmap_buildy
                );
                Canvas::update(&mta.pos, |x: i32, y: i32, p: Vec3| {
                    m.draw_at(x, y, p);
                });
            }
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    });
}

fn on_event() {}
