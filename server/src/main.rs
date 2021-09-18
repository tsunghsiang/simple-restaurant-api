mod cmd;
mod db;
mod order_type;
mod settings;

use cmd::Dbio;
use ctrlc;
use db::DB;
use lazy_static::lazy_static;
use order_type::DeleteOrder;
use order_type::PlaceOrder;
use order_type::UpdateOrder;
use settings::Settings;
use sha256::digest_bytes;
use std::process;
use std::sync::Mutex;
use std::{thread, time};

#[macro_use]
extern crate postgres;
#[macro_use]
extern crate postgres_derive;

struct Signal {
    terminate: bool,
}

impl Signal {
    fn new(signal: bool) -> Signal {
        Signal { terminate: signal }
    }

    fn set(&mut self, signal: bool) {
        self.terminate = signal;
    }

    fn get(&self) -> bool {
        self.terminate
    }
}

lazy_static! {
    static ref SIGNAL: Mutex<Signal> = Mutex::new(Signal::new(false));
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut server = tide::new();
    let mut host: String = "".to_string();
    let config: Settings = Settings::new();
    host.push_str(&config.server.get_ip());
    host.push_str(":");
    host.push_str(&config.server.get_port());
    let command: Dbio = Dbio::new();

    /* Check DB status first */
    match command.init() {
        Ok(()) => println!("[DATABASE] DB status OK"),
        Err(err) => {
            println!("[DATBASE] DB Error: {}", err);
            panic!("[DATABASE] Please check db connection status.");
        }
    };

    //ctrlc::set_handler(error_handler).expect("Error setting Ctrl-C handler");
    tide::log::start();

    /* simple api processing here */
    server
        .at("/api/status/order/:tableid")
        .get(query_by_tableid);
    server
        .at("/api/status/order/:tableid/:item")
        .get(query_by_tableid_and_item);
    server.at("/api/place/order").post(add_by_tableid_and_item);
    server
        .at("/api/delete/order")
        .delete(remove_by_tableid_and_item);
    server
        .at("/api/update/order")
        .patch(update_by_tableid_and_item);
    server.listen(host).await?;

    Ok(())
}

/*
fn error_handler() {
    println!("[TERMINATION] Received signal to terminate the server!");
    /* Waiting incomplete requests to be done */
    SIGNAL.lock().unwrap().set(true);
    let command: Dbio = Dbio::new();
    let mut all_done: bool = false;

    loop {
        match command.check_table_status() {
            Ok(done) => {
                all_done = done;
            }
            _ => {}
        }
        if all_done {
            break;
        } else {
            println!("[TERMINATION] Waiting for tables to be fully served");
            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    println!("[TERMINATION] Fully served! Server terminated.");
    process::exit(0);
}
*/
fn is_auth(req: &tide::Request<()>) -> bool {
    (match req.header("X-Auth-Username") {
        Some(name) => {
            let username = b"restaurant";
            if name.as_str() == digest_bytes(username) {
                // println!("X-Auth-Username: {}", name.as_str());
                true
            } else {
                false
            }
        }
        None => false,
    }) && (match req.header("X-Auth-Password") {
        Some(pwd) => {
            let password = b"paidy";
            if pwd.as_str() == digest_bytes(password) {
                // println!("X-Auth-Password: {}", pwd.as_str());
                true
            } else {
                false
            }
        }
        None => false,
    })
}

async fn query_by_tableid(req: tide::Request<()>) -> tide::Result {
    let terminated: bool = SIGNAL.lock().unwrap().get();
    if !terminated {
        let mut collection = req.url().as_str().split('/');
        let mut res: String = "".to_string();
        let command: Dbio = Dbio::new();
        let table_id = collection.nth_back(0).unwrap();

        match command.query_by_tableid(table_id.to_string()) {
            Ok(result) => res = result,
            _ => {}
        };

        Ok(res.into())
    } else {
        Ok("Server is Closing. No More Services".into())
    }
}

async fn query_by_tableid_and_item(req: tide::Request<()>) -> tide::Result {
    let terminated: bool = SIGNAL.lock().unwrap().get();
    if !terminated {
        let mut collection = req.url().as_str().split('/');
        let mut res: String = "".to_string();
        let command: Dbio = Dbio::new();
        let item = collection.nth_back(0).unwrap();
        let table_id = collection.nth_back(0).unwrap();

        match command.query_by_tableid_and_item(table_id.to_string(), item.to_string()) {
            Ok(result) => res = result,
            _ => {}
        };

        Ok(res.into())
    } else {
        Ok("Server is Closing. No More Services".into())
    }
}

async fn add_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let terminated: bool = SIGNAL.lock().unwrap().get();
    if !terminated {
        if is_auth(&req) {
            let order: PlaceOrder = req.body_json().await?;
            //    let command: Dbio = Dbio::new();
            //    let mut res: String = "".to_string();
            //    match command.place(order) {
            //        Ok(result) => res = result,
            //        _ => {}
            //    };
            //    Ok(res.into())
            Ok(order.disp().into())
        } else {
            Ok("Un-authorized place order".into())
        }
    } else {
        Ok("Server is Closing. No More Services".into())
    }
}

async fn remove_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let terminated: bool = SIGNAL.lock().unwrap().get();
    if !terminated {
        if is_auth(&req) {
            let order: DeleteOrder = req.body_json().await?;
            //let command: Dbio = Dbio::new();
            //let mut res: String = "".to_string();
            //match command.delete(order) {
            //    Ok(result) => res = result,
            //    _ => {}
            //};
            //Ok(res.into())
            Ok(order.disp().into())
        } else {
            Ok("Un-authorized delete order".into())
        }
    } else {
        Ok("Server is Closing. No More Services".into())
    }
}

async fn update_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let terminated: bool = SIGNAL.lock().unwrap().get();
    if !terminated {
        if is_auth(&req) {
            let order: UpdateOrder = req.body_json().await?;
            //let command: Dbio = Dbio::new();
            //let mut res: String = "".to_string();
            //match command.update(order) {
            //    Ok(result) => res = result,
            //    _ => {}
            //};
            //Ok(res.into())
            Ok(order.disp().into())
        } else {
            Ok("Un-authorized update order".into())
        }
    } else {
        Ok("Server is Closing. No More Services".into())
    }
}
