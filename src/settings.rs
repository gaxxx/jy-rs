use std::env;
use std::str::FromStr;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use bevy::log::Level;
use bevy::prelude::{FromWorld, World};

#[derive(Debug, Deserialize)]
pub struct Settings {
    // ["off", "error", "warn", "info", "debug", "trace"];
    log: String,
}

impl FromWorld for Settings {
    fn from_world(_: &mut World) -> Self {
        #[cfg(target_arch = "wasm32")]
        return Settings {
            log: "error".into(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        match Settings::load() {
            Ok(s) => s,
            Err(e) => {
                println!("failed to load from config: {:?}\n, load default", e);
                Settings {
                    log: "error".into(),
                }
            }
        }
    }
}

impl Settings {
    pub fn log_level(&self) -> Level {
        Level::from_str(&self.log).unwrap_or(Level::ERROR)
    }

    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

        let s = Config::new()
            // Start off by merging in the "default" configuration file
            .with_merged(File::with_name("conf/settings").required(false))?
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .with_merged(File::with_name(&format!("conf/settings_{}", run_mode)).required(false))?
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .with_merged(Environment::with_prefix("app"))?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
