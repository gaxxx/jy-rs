use super::*;


pub fn instruct_1(talk_id: i32, head_id: i32, flag: i32) -> JyEvent {
    JyEvent::Dialog("Hello world".into())
}

pub fn handle_instruct_1(
        mut commands: Commands,
        mut mb_ev_script: Option<ResMut<EventScript>>,
        asset_server: Res<AssetServer>,
        ) {
    if mb_ev_script.is_none() {
        return;
    }

    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(JyEvent::Dialog(v)) = ev_script.dispatch.as_ref() {
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
        ev_script.wait_input = true;
        ev_script.dispatch.take();
    }
}