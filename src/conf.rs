use jsonschema::*;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigHideBrowsers {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub version: i16,

    #[serde(default)]
    pub default_url: String,

    #[serde(default)]
    pub hide: Vec<ConfigHideBrowsers>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: 1,
            hide: Default::default(),
            default_url: String::from("about:home"),
        }
    }
}

pub fn read_config() -> Option<Config> {
    let schema = serde_yaml::from_str(include_str!("data/conf.schema.yaml")).unwrap_or_default();
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("Invalid config schema.");

    let mut config_path = std::env::current_exe().unwrap_or_default();
    config_path.set_file_name("config.yml");

    let file = std::fs::File::open(config_path.clone());
    if file.is_err() {
        return None;
    }

    let mut config_contents = String::new();
    let config_yaml: serde_json::Value = match file.unwrap().read_to_string(&mut config_contents) {
        Ok(0) | Err(_) => return None,
        Ok(_) => serde_yaml::from_str(&config_contents).expect("Unable to parse config YAML"),
    };

    match compiled_schema.validate(&config_yaml) {
        Err(errors) => panic!(
            "{}",
            errors.fold(String::from("Validation errors:\n"), |mut acc, item| {
                acc.push_str(&item.to_string());
                acc
            },)
        ),
        Ok(_) => true,
    };

    let conf: Result<Config, _> = serde_json::from_value(config_yaml);
    if conf.is_err() {
        println!(
            "Ignoring config due to error:\n{}",
            conf.err().unwrap().to_string()
        );
        return None;
    }

    return Some(conf.unwrap());
}
