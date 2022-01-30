#![feature(default_free_fn)]

use bevy::prelude::*;
use jy::prelude::*;

fn main() {
    let mut main_app = App::new();
    let settings = Settings::new();
    main_app
        .insert_resource(WindowDescriptor {
            title: "JY!".to_string(),
            width: 1024.,
            height: 768.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(settings)
        .add_plugins(DefaultPlugins)
        .add_plugin(AdminPlugin)
        .add_plugin(GamePlugin)
        .run()
    ;

    main_app.run();
}
