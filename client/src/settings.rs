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
    pub fn new(url: String, timeout: u64) -> Client {
        Client {
            url: url,
            timeout: timeout,
        }
    }
    pub fn get_base_url(&self) -> String {
        self.url.clone()
    }
    pub fn get_timeout(&self) -> u64 {
        self.timeout
    }
}

impl API {
    pub fn new(
        place_order_api: String,
        delete_order_api: String,
        update_order_api: String,
        status_order_api: String,
    ) -> API {
        API {
            place_order: place_order_api,
            delete_order: delete_order_api,
            update_order: update_order_api,
            status_order: status_order_api,
        }
    }
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
    pub fn new(username: String, password: String) -> Auth {
        Auth {
            username: username,
            password: password,
        }
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }
    pub fn get_password(&self) -> String {
        self.password.clone()
    }
}

impl Settings {
    pub fn new(client: Client, api: API, auth: Auth) -> Settings {
        Settings {
            client: client,
            api: api,
            auth: auth,
        }
    }
}
