use bevy::prelude::*;
use jy::prelude::*;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    let mut main_app = App::new();
    main_app
        .insert_resource(WindowDescriptor {
            title: "JY!".to_string(),
            width: 1024.,
            height: 768.,
            ..Default::default()
        })
        .add_plugin(AdminPlugin)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();

    main_app.run();
}
