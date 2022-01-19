use bevy::prelude::*;
use jy::admin;
use jy::Settings;

fn main() {
    let mut main_app = App::new();
    let settings = Settings::new();
    main_app
        .insert_resource(settings)
        .add_plugins(DefaultPlugins)
        .add_plugin(admin::Plugin)
    ;

    main_app.run();
}
