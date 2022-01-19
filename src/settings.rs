use std::env;
use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub debug : bool
}

impl Settings {
    pub fn new() -> Self {
        match Settings::load() {
            Ok(s) => s,
            Err(e) => {
                println!("failed to load from config: {:?}\n, load default", e);
                Settings::default()
            }
        }
    }

    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

        let s = Config::new()
            // Start off by merging in the "default" configuration file
            .with_merged(File::with_name("conf/settings").required(false))?
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .with_merged(
                File::with_name(&format!("conf/settings_{}", run_mode))
                    .required(false),
            )?
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .with_merged(Environment::with_prefix("app"))?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}