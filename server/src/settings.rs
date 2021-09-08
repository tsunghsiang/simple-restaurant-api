use config::{Config, File};
use serde::Deserialize;

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
        match config.merge(File::with_name("server/config/production.toml")) {
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
