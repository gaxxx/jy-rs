use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::{Level, LogSettings};
use bevy::prelude::*;

use crate::settings::Settings;

pub struct Plugin;

fn setup() {
    info!("hello admin");
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>();
        let settings = app.world.get_resource::<Settings>().unwrap();
        let level = settings.log_level();
        app.insert_resource(LogSettings {
            level: level.clone(),
            filter: "gilrs=error,wgpu=error,bevy_render=warn,bevy_app=error,naga=error".to_string(),
        });
        if level >= Level::TRACE {
            app.add_plugin(DiagnosticsPlugin)
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(FrameTimeDiagnosticsPlugin::default());
        }
        app.add_startup_system(setup);
    }
}
