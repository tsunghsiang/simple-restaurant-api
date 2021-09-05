use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Client {
    url: String,
}

#[derive(Debug, Deserialize)]
struct API {
    place_order: String,
    delete_order: String,
    update_order: String,
    status_order: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    client: Client,
    api: API,
}

impl Settings {
    pub fn get_base_url() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("client.base_url") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }

    pub fn get_place_order_api() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("api.place_order") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }

    pub fn get_delete_order_api() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("api.delete_order") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }

    pub fn get_update_order_api() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("api.update_order") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }

    pub fn get_status_order_api() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("api.status_order") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }
}
