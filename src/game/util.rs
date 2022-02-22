#![allow(dead_code)]

use core::default::Default;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;

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
        Self {
            image_cache,
            sta,
            s_data,
            d_data,
            meshes,
            materials,
        }
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        debug!("despawn entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}
