use super::*;

pub fn instruct_0() -> JyEvent {
    JyEvent::Cls
}

pub fn handle_instruct_0(
        mut commands: Commands,
        mut mb_ev_script: Option<ResMut<EventScript>>,
        query: Query<Entity, With<DialogBox>>,
        asset_server: Res<AssetServer>,
        ) {
    if mb_ev_script.is_none() {
        return;
    }

    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(&JyEvent::Cls) = ev_script.dispatch.as_ref() {
        ev_script.dispatch.take();
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}