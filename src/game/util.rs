#![allow(dead_code)]

use core::default::Default;

use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::game::structs::*;

#[derive(Default)]
pub struct Status {
    pub cur_ev: i16,
    pub cur_s: i32,
    pub cur_s_x: i16,
    pub cur_s_y: i16,
    pub cur_d: (usize, usize, usize),
    pub cur_pic: usize,
    pub is_new_game: bool,
}

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
    ) -> (Handle<Image>, TextureMeta, Option<&'a Image>) {
        if let Some((h, meta)) = self.cached.get(&id) {
            (h.clone(), *meta, self.assets.get(h))
        } else if let Some((image, meta)) = self.smap.get_image(id, &self.palette.0) {
            let handle = self.assets.add(image);
            self.cached.insert(id, (handle.clone(), meta.clone()));
            (handle.clone(), meta, self.assets.get(handle))
        } else {
            panic!("no image here, the asset id: {} is wrong", id)
        }
    }
}
