use bevy::prelude::*;

use crate::game::mmap::MMapStatus;
use crate::game::structs::{SData, SceneStatus, MapType};
use crate::game::util::ImageCache;
use crate::game::GameState;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_hint_box)
            .add_system(update_hint_box);
    }
}

#[derive(Component)]
pub struct HintInfo;

fn setup_hint_box(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let entity = commands
        .spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: format!("cur_x\n"),
                        style: TextStyle {
                            font: asset_server.load("fonts/simsun.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("cur_y\n"),
                        style: TextStyle {
                            font: asset_server.load("fonts/simsun.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("block\n"),
                        style: TextStyle {
                            font: asset_server.load("fonts/simsun.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: format!("meta\n"),
                        style: TextStyle {
                            font: asset_server.load("fonts/simsun.ttf"),
                            font_size: 12.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                alignment: Default::default(),
            },
            transform: Transform::from_xyz(300.0, 300., 10.),
            ..Default::default()
        })
        .id();
    commands.entity(entity.clone()).insert(HintInfo);
}

fn update_hint_box(
    mut query: Query<(&mut Text, &HintInfo)>,
    mb_image_cache: Option<ResMut<ImageCache>>,
    state: Res<State<GameState>>,
    mb_mta: Option<Res<MMapStatus>>,
    mb_sdata: Option<Res<SData>>,
    mb_sta: Option<Res<SceneStatus>>,
) {
    let mut mb_meta = None;
    match state.current() {
        GameState::Smap => {
            if let (Some(sdata), Some(sta), Some(mut image_cache)) =
                (mb_sdata, mb_sta, mb_image_cache)
            {
                let pic = sdata.get_texture(
                    sta.cur_s as usize,
                    sta.pos.x as usize,
                    sta.pos.y as usize,
                    1,
                );
                if pic > 0 {
                    mb_meta = image_cache.get_image(MapType::Smap, pic as usize / 2).map(|v| v.1.clone());
                }
                let (mut text, _) = query.single_mut();
                text.sections[0].value = format!("cur_x : {}\n", sta.pos.x as i16);
                text.sections[1].value = format!("cur_y : {}\n", sta.pos.y as i16);
                text.sections[2].value = format!("block : {}\n", pic);
                text.sections[3].value = format!(
                    "meta: {}\n",
                    mb_meta
                        .map(|v| { format!("w{}h{}:{}:{}", v.0, v.1, v.2, v.3) })
                        .unwrap_or("".into())
                );
            }
        }
        GameState::Mmap => {
            if let Some(mta) = mb_mta {
                let (mut text, _) = query.single_mut();
                text.sections[0].value = format!("cur_x : {}\n", mta.pos.x as i16);
                text.sections[1].value = format!("cur_y : {}\n", mta.pos.y as i16);
                text.sections[2].value = format!("\n");
                text.sections[3].value = format!("\n");
            }
        }
        _ => {}
    }
}
