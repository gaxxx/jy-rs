

use super::*;

pub fn instruct_2(thing: i16, num: i16) -> JyEvent {
    JyEvent::Instruct2(thing, num)
}

pub fn handle_instruct_2(
        d_data: ResMut<DData>,
        things: Res<Vec<Thing>>,
        mut backpack: ResMut<Backpack>,
        mut mb_ev_script: Option<ResMut<EventScript>>,
        sta: Res<SceneStatus>,
        ) {
    if mb_ev_script.is_none() {
        return;
    }
    let ev_script = mb_ev_script.as_mut().unwrap();
    if let Some(&JyEvent::Instruct2(thing, size)) = ev_script.dispatch.as_ref() {
        add_to_backpack(&mut backpack, thing, size);
        ev_script.dispatch.take();
        let output = format!("得到物品:{} {}", things[thing as usize].name(), size);
        ev_script.events.push(JyEvent::Dialog(output));
        ev_script.events.push(JyEvent::Cls);
    }
}

fn add_to_backpack(backpack: &mut ResMut<Backpack>, thing: i16, size: i16) {
    match backpack
    .items
    .iter_mut()
    .enumerate()
    .find(|(_, (item, _))| *item == thing)
    {
        None => {
            backpack.items.push((thing, size));
        }
        Some((idx, (item, cur))) => {
            *cur += size;
            if *cur == 0 {
                backpack.items.remove(idx);
            }
        }
    }
}