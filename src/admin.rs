use bevy::{prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::settings::Settings;

pub struct Plugin;

fn setup() {
    info!("hello admin");
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let settings = app.world.get_resource::<Settings>().unwrap();
        if settings.log_level() >= log::Level::Trace {
            app
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(FrameTimeDiagnosticsPlugin::default());
        }
        app.add_startup_system(setup);
    }
}
