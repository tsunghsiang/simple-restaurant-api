use config::{Config, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Server {
    ip: String,
    port: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    prefix: String,
    password: String,
    ip: String,
    port: String,
    db_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
}

impl Server {
    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }
    pub fn get_port(&self) -> String {
        self.port.clone()
    }
}

impl Database {
    pub fn get_prefix(&self) -> String {
        self.prefix.clone()
    }
    pub fn get_password(&self) -> String {
        self.password.clone()
    }
    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }
    pub fn get_port(&self) -> String {
        self.port.clone()
    }
    pub fn get_db_name(&self) -> String {
        self.db_name.clone()
    }
}

impl Settings {
    pub fn new() -> Self {
        let mut config: Config = Config::default();
        let (mut ip, mut port) = ("".to_string(), "".to_string());
        let (mut prefix, mut password, mut db_ip, mut db_port, mut db_name) = (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );

        let relative_path: PathBuf;
        let mut absolute_path = std::env::current_dir().unwrap();
        let mut path: &str = "";

        //println!("absolute path: {:#?}", absolute_path);
        if absolute_path.ends_with("server") {
            relative_path = PathBuf::from("config\\production.toml");
        } else {
            relative_path = PathBuf::from("server\\config\\production.toml");
        };
        absolute_path.push(relative_path);

        match absolute_path.to_str() {
            Some(field) => path = field,
            None => {}
        };
        match config.merge(File::with_name(path)) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        }
        match config.get::<String>("server.ip") {
            Ok(field) => ip = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("server.port") {
            Ok(field) => port = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("database.prefix") {
            Ok(field) => prefix = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("database.password") {
            Ok(field) => password = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("database.ip") {
            Ok(field) => db_ip = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("database.port") {
            Ok(field) => db_port = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }
        match config.get::<String>("database.db_name") {
            Ok(field) => db_name = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        }

        Settings {
            server: Server { ip: ip, port: port },
            database: Database {
                prefix: prefix,
                password: password,
                ip: db_ip,
                port: db_port,
                db_name: db_name,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_settings_new_given_config_provided_when_init_then_fields_obtained() {
        let config: Settings = Settings::new();
        assert!(config.server.get_ip().len() > 0);
        assert!(config.server.get_port().len() > 0);
        assert_eq!("postgresql://postgres", config.database.get_prefix());
        assert!(config.database.get_ip().len() > 0);
        assert!(config.database.get_port().len() > 0);
        assert_eq!("restaurant", config.database.get_db_name());
    }
}
