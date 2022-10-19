use crate::game::{util::RenderHelper, smap::{JyBox, SMapScreen}};

use super::*;

pub fn instruct_3(
        s: i16,
        d: i16,
        v0: i16,
        v1: i16,
        v2: i16,
        v3: i16,
        v4: i16,
        v5: i16,
        v6: i16,
        v7: i16,
        v8: i16,
        v9: i16,
        v10: i16,
        ) -> JyEvent {
    let v = vec![v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10];
    let t = v
    .into_iter()
    .enumerate()
    .fold(vec![], |mut acc, (idx, val)| {
        if val != -2 {
            acc.push((idx, val));
        }
        acc
    });

    JyEvent::Data(s, d, t)
}

pub fn handle_instruct_3(
        mut commands: Commands,
        mut d_data: ResMut<DData>,
        mut mb_ev_script: Option<ResMut<EventScript>>,
        sta: ResMut<SceneStatus>,
        mut render_helper: ResMut<RenderHelper>,
        s_data: Res<SData>,
        query: Query<(&Transform, &JyBox), With<JyBox>>,
        ) {
    if mb_ev_script.is_none() {
        return;
    }
    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(JyEvent::Data(sc, d, vals)) = ev_script.dispatch.as_ref() {
        let s = if *sc == -2i16 {
            sta.cur_s
        } else {
            *sc as usize
        };
        let dv = if *d == -2i16 {
            sta.cur_d.0
        } else {
            *d as usize
        };

        let mut image_update = None;
        for (k, v) in vals {
            d_data.set(s as usize, dv, *k, *v);
            if *k == 7 {
                image_update = Some(*v);
            }
        }

        if image_update.is_some() {
            for (tran, bx) in query.iter() {
                if bx.1 == sta.cur_d.1 && bx.2 == sta.cur_d.2 {
                    let x = bx.1 as f32;
                    let y = bx.2 as f32;
                    commands.entity(bx.0).despawn_recursive();
                    let mut trans = Transform::from_translation(sta.pos.to_real(x, y, 3.));
                    trans.translation.y +=
                    s_data.get_texture(sta.cur_s, x as usize, y as usize, 4) as f32;
                    render_helper
                    .render(
                            &mut commands,
                    MapType::Smap,
                    image_update.clone().unwrap() as usize / 2,
                    trans,
                    )
                    .map(|v| {
                        commands.entity(v).insert(SMapScreen);
                    });
                }
            }
        }

        ev_script.dispatch.take();
    }
}