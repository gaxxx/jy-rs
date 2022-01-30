use bevy::audio::PlayEvent;
use bevy::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(play_audio);
    }
}

fn play_audio(_commands: Commands, res: Res<AssetServer>, mut ew: EventWriter<PlayEvent>) {
    let cur = "sounds/game01.mp3";
    let sound = res.load::<AudioSource, &'static str>(cur);
    ew.send(PlayEvent::Loop(true));
    ew.send(PlayEvent::Append(sound));
}
