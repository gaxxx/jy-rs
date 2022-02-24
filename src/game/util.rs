#![allow(dead_code)]

use core::default::Default;
use std::ops::Deref;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;

use crate::game::script::SpriteMeta;
use crate::game::smap::NetCell;
use crate::game::structs::*;

pub struct ImageCache {
    pub cached: HashMap<usize, (Handle<Image>, TextureMeta)>,
    pub smap: &'static TextureMap,
    pub palette: &'static Palette,
    pub assets: &'static mut Assets<Image>,
}

impl FromWorld for ImageCache {
    fn from_world(world: &mut World) -> Self {
        let text_map = world.get_resource::<TextureMap>();
        let palette = unsafe { std::mem::transmute(world.get_resource::<Palette>().unwrap()) };
        let smap = unsafe { std::mem::transmute(text_map.unwrap()) };
        let assets = unsafe {
            std::mem::transmute(world.get_resource_mut::<Assets<Image>>().unwrap().as_mut())
        };
        Self {
            cached: HashMap::default(),
            smap,
            palette,
            assets,
        }
    }
}

impl ImageCache {
    pub fn get_image<'a>(
        &'a mut self,
        id: usize,
    ) -> Option<(Handle<Image>, TextureMeta, Option<&'a Image>)> {
        if let Some((h, meta)) = self.cached.get(&id) {
            Some((h.clone(), *meta, self.assets.get(h)))
        } else if let Some((image, meta)) = self.smap.get_image(id, &self.palette.0) {
            let handle = self.assets.add(image);
            self.cached.insert(id, (handle.clone(), meta.clone()));
            Some((handle.clone(), meta, self.assets.get(handle)))
        } else {
            println!("no image here, the asset id: {} is wrong", id);
            None
        }
    }
}

pub struct RenderHelper {
    image_cache: &'static mut ImageCache,
    sta: &'static SceneStatus,
    s_data: &'static SData,
    d_data: &'static DData,
    meshes: &'static mut Assets<Mesh>,
    materials: &'static mut Assets<ColorMaterial>,
    textures: &'static mut Assets<TextureAtlas>,
}

impl RenderHelper {
    pub fn render(
        &mut self,
        commands: &mut Commands,
        pic_id: usize,
        mut transform: Transform,
    ) -> Option<Entity> {
        if let Some((image_h, meta, _)) = self.image_cache.get_image(pic_id as usize) {
            transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
            transform.translation.y += meta.3 - meta.1 as f32 / 2.;
            Some(
                commands
                    .spawn_bundle(MaterialMesh2dBundle {
                        mesh: self.meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: transform.with_scale(Vec3::new(
                            meta.0 as f32,
                            meta.1 as f32,
                            0.,
                        )),
                        material: self.materials.add(ColorMaterial::from(image_h)),
                        ..Default::default()
                    })
                    .insert(NetCell)
                    .id(),
            )
        } else {
            None
        }
    }

    pub fn render_sprite(&mut self, commands: &mut Commands, images: &mut Assets<Image>) -> Entity {
        let cur_pic = 2501;
        let mut texture_builder =
            TextureAtlasBuilder::default().initial_size(Vec2::new(XSCALE * 2., YSCALE * 2.));
        let mut metas = vec![];
        (0..28).for_each(|v| {
            if let Some((image_h, meta, Some(image))) = self.image_cache.get_image(cur_pic + v) {
                metas.push(meta);
                texture_builder.add_texture(image_h, &image);
            }
        });
        let texture = texture_builder.finish(images).unwrap();
        let texture_atlas = self.textures.add(texture);
        let mut transform = Transform::from_xyz(0., 0., 3.);
        let meta = metas[0];
        transform.translation.x -= meta.2 - meta.0 as f32 / 2.;
        // add y scale to make the offset right, I don't know fucking why
        transform.translation.y += YSCALE + meta.3 - meta.1 as f32 / 2.;
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas,
                transform,
                ..Default::default()
            })
            .insert(SpriteMeta(metas, Timer::from_seconds(0.5, true)))
            .id()
    }
}

impl FromWorld for RenderHelper {
    fn from_world(world: &mut World) -> Self {
        let image_cache = unsafe {
            std::mem::transmute(world.get_resource_mut::<ImageCache>().unwrap().as_mut())
        };
        let sta = unsafe { std::mem::transmute(world.get_resource::<SceneStatus>().unwrap()) };
        let s_data = unsafe { std::mem::transmute(world.get_resource::<SData>().unwrap()) };
        let d_data = unsafe { std::mem::transmute(world.get_resource::<DData>().unwrap()) };
        let meshes = unsafe {
            std::mem::transmute(world.get_resource_mut::<Assets<Mesh>>().unwrap().as_mut())
        };
        let materials = unsafe {
            std::mem::transmute(
                world
                    .get_resource_mut::<Assets<ColorMaterial>>()
                    .unwrap()
                    .as_mut(),
            )
        };
        let textures = unsafe {
            std::mem::transmute(
                world
                    .get_resource_mut::<Assets<TextureAtlas>>()
                    .unwrap()
                    .as_mut(),
            )
        };
        Self {
            image_cache,
            sta,
            s_data,
            d_data,
            meshes,
            materials,
            textures,
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

trait PosTrans {
    fn to_real(&self, x: f32, y: f32) -> (f32, f32);
    fn to_related(&self, x: f32, y: f32) -> (f32, f32);
}

#[derive(Default, Debug)]
pub struct PosXY {
    val: Vec2,
    x_off: f32,
    y_off: f32,
    pub facing: Option<MoveDir>,
}

impl Deref for PosXY {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl PosXY {
    pub fn new(x: usize, y: usize) -> Self {
        let mut v = Self::default();
        v.facing = Some(MoveDir::Up);
        v.update(x, y);
        v
    }

    pub fn update(&mut self, x: usize, y: usize) {
        self.val.x = x as f32;
        self.val.y = y as f32;
        self.x_off = (self.y - self.x) * XSCALE;
        self.y_off = (self.x + self.y) * YSCALE;
    }

    pub fn facing(&self) -> &MoveDir {
        self.facing.as_ref().unwrap()
    }

    pub fn to_real(&self, x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(
            (x - y) * XSCALE + self.x_off,
            (-x - y) * YSCALE + self.y_off,
            z,
        )
    }

    // from:
    // a = (x - y) * 18 + x_off;
    // b = (-x - y) * 9 + y_off;
    // to
    // x = (a - b * 2 + y_off *2  - x_off) / 36;
    // y = (x_off + y_off * 2 - a - b * 2) / 36;
    pub fn to_related(&self, a: f32, b: f32) -> (i32, i32) {
        (
            ((a - b * 2. + self.y_off * 2. - self.x_off) / 36.) as i32,
            ((self.x_off + self.y_off * 2. - a - b * 2.) / 36.) as i32,
        )
    }
}

const X_MAX: f32 = 600.;
const Y_MAX: f32 = 400.;

pub struct Canvas;

impl Canvas {
    pub fn update<T>(init: &PosXY, mut func: T)
    where
        T: FnMut(i32, i32, Vec3),
    {
        let (x_tl, y_tl) = init.to_related(-X_MAX, Y_MAX);
        let (x_bl, y_bl) = init.to_related(-X_MAX, -Y_MAX);
        trace!("tl {}:{} bl {}:{}", x_tl, y_tl, x_bl, y_bl);
        trace!(
            "real tl {:?}, real bl {:?}",
            init.to_real(x_tl as f32, y_tl as f32, 0.),
            init.to_real(x_bl as f32, y_bl as f32, 0.),
        );
        //      (x,y) ---- (x+1, y-1) -- (x+2, y-2)
        // (x,y+1)
        //      (x+1, y+1)--(x+2,y)
        // (x+1,y+2)

        // from first row to last row
        let row = (Y_MAX * 2. / YSCALE) as i32 + 1;
        let col = (X_MAX / XSCALE) as i32 + 1;

        for r_idx in 0..row {
            let (x_start, y_start) = if r_idx % 2 == 0 {
                (x_tl + r_idx / 2, y_tl + r_idx / 2)
            } else {
                (x_tl + r_idx / 2, y_tl + 1 + r_idx / 2)
            };
            for c_idx in 0..col {
                let x = x_start + c_idx;
                let y = y_start - c_idx;

                func(x, y, init.to_real(x as f32, y as f32, 0.));
            }
        }
    }
}
