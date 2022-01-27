#![feature(default_free_fn)]

use bevy::prelude::*;
use jy::prelude::*;

fn main() {
    let mut main_app = App::new();
    let settings = Settings::new();
    main_app
        .insert_resource(settings)
        .add_plugins(DefaultPlugins)
        .add_plugin(AdminPlugin)
        .add_plugin(GamePlugin)
        .run()
    ;

    main_app.run();
}
