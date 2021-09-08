mod order_type;
mod settings;
mod staff;
mod tablet;

use config::{Config, File};
use settings::{Auth, Client, Settings, API};
use staff::Staff;
use std::env;
use std::panic;
use tablet::Tablet;
use threadpool::ThreadPool;
extern crate time;

#[tokio::main]
async fn main() {
    // Start staffs to serve customers
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("[ERR] Please specify number of staffs: cargo run --bin client [num], [num] should be a positive integer");
    }
    let nums = (&args[1]).parse().unwrap();
    println!("Amount of staffs spawned: {}", nums);

    let pool = ThreadPool::with_name("staff-group".into(), nums);
    for i in 1..nums + 1 {
        pool.execute(move || {
            //let auth: Auth = Settings::get_auth();
            let config: Settings = Settings::new(get_client(), get_api(), get_auth());
            let staff: Staff = Staff::new(i.to_string(), config);
            println!(
                "------------------------- [SPAWN-STAFF-ID] {} -------------------------",
                staff.get_table_id()
            );
            staff.work();
        });
    }
    pool.join();
}

fn get_client() -> Client {
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
    Client::new(url, timeout)
}

fn get_api() -> API {
    let mut config: Config = Config::default();
    let (mut place_order_api, mut delete_order_api, mut update_order_api, mut status_order_api) = (
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );
    match config.merge(File::with_name("client/config/production.toml")) {
        Ok(_) => {}
        Err(err) => println!("[SETTINGS] Config Error: {}", err),
    };
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
    API::new(
        place_order_api,
        delete_order_api,
        update_order_api,
        status_order_api,
    )
}

fn get_auth() -> Auth {
    let mut config: Config = Config::default();
    let (mut uname, mut pwd) = ("".to_string(), "".to_string());
    match config.merge(File::with_name("client/config/production.toml")) {
        Ok(_) => {}
        Err(err) => println!("[SETTINGS] Config Error: {}", err),
    };
    match config.get::<String>("auth.username") {
        Ok(field) => uname = field.to_string(),
        Err(err) => println!("[SETTINGS] Error: {}", err),
    };
    match config.get::<String>("auth.password") {
        Ok(field) => pwd = field.to_string(),
        Err(err) => println!("[SETTINGS] Error: {}", err),
    };
    Auth::new(uname, pwd)
}
