use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Client {
    url: String,
    timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct API {
    place_order: String,
    delete_order: String,
    update_order: String,
    status_order: String,
}

#[derive(Debug, Deserialize)]
pub struct Auth {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub client: Client,
    pub api: API,
    pub auth: Auth,
}

impl Client {
    pub fn get_base_url(&self) -> String {
        self.url.clone()
    }
    pub fn get_timeout(&self) -> u64 {
        self.timeout
    }
}

impl API {
    pub fn get_place_order_api(&self) -> String {
        self.place_order.clone()
    }
    pub fn get_delete_order_api(&self) -> String {
        self.delete_order.clone()
    }
    pub fn get_update_order_api(&self) -> String {
        self.update_order.clone()
    }
    pub fn get_status_order_api(&self) -> String {
        self.status_order.clone()
    }
}

impl Auth {
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
    pub fn get_password(&self) -> String {
        self.password.clone()
    }
}

impl Settings {
    pub fn new() -> Self {
        let mut config: Config = Config::default();
        let (mut url, mut timeout) = ("".to_string(), 0);
        match config.merge(File::with_name("client/config/production.toml")) {
            Ok(_) => {}
            Err(err) => println!("[SETTINGS] Config Error: {}", err),
        };
        match config.get::<String>("client.base_url") {
            Ok(field) => url = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };
        match config.get::<u64>("client.timeout") {
            Ok(field) => timeout = field,
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };

        let (mut place_order_api, mut delete_order_api, mut update_order_api, mut status_order_api) = (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        match config.get::<String>("api.place_order") {
            Ok(field) => place_order_api = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };
        match config.get::<String>("api.delete_order") {
            Ok(field) => delete_order_api = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };
        match config.get::<String>("api.update_order") {
            Ok(field) => update_order_api = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };
        match config.get::<String>("api.status_order") {
            Ok(field) => status_order_api = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };

        let (mut uname, mut pwd) = ("".to_string(), "".to_string());
        match config.get::<String>("auth.username") {
            Ok(field) => uname = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };
        match config.get::<String>("auth.password") {
            Ok(field) => pwd = field.to_string(),
            Err(err) => println!("[SETTINGS] Error: {}", err),
        };

        Settings {
            client: Client {
                url: url,
                timeout: timeout,
            },
            api: API {
                place_order: place_order_api,
                delete_order: delete_order_api,
                update_order: update_order_api,
                status_order: status_order_api,
            },
            auth: Auth {
                username: uname,
                password: pwd,
            },
        }
    }
}
