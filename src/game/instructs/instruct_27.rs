use crate::game::{smap::{Me, SMapScreen}, mmap::ImageCache, util::RenderHelper};

use super::*;

pub fn instruct_27(id: i16, start_pic: i16, end_pic: i16) -> JyEvent {
    JyEvent::Sprite
}

pub fn handle_instruct_27(
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