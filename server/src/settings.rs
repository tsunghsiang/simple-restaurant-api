use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Server {
    ip: String,
    port: String,
}

#[derive(Debug, Deserialize)]
struct Database {
    prefix: String,
    password: String,
    ip: String,
    port: String,
    db_name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    server: Server,
    database: Database,
}

impl Settings {
    pub fn get_db_url() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("server/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("database.prefix") {
            Ok(field) => {
                res.push_str(&field);
                res.push_str(":");
            }
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };

        match config.get::<String>("database.password") {
            Ok(field) => {
                res.push_str(&field);
                res.push_str("@");
            }
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        match config.get::<String>("database.ip") {
            Ok(field) => {
                res.push_str(&field);
                res.push_str(":");
            }
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        match config.get::<String>("database.port") {
            Ok(field) => {
                res.push_str(&field);
                res.push_str("/");
            }
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        match config.get::<String>("database.db_name") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }

    pub fn get_srv_url() -> String {
        let mut config: Config = Config::default();
        let mut res: String = "".to_string();
        match config.merge(File::with_name("server/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }

        match config.get::<String>("server.ip") {
            Ok(field) => {
                res.push_str(&field);
                res.push_str(":");
            }
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("server.port") {
            Ok(field) => res.push_str(&field),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        res
    }
}
