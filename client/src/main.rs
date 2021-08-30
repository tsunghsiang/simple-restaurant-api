mod tablet;
mod staff;
mod order_type;

use tablet::Tablet;
use staff::Staff;
use std::env;
use std::panic;
use threadpool::ThreadPool;
use reqwest::Client;
//use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
    for i in 1..nums+1 {
        pool.execute(move || {
            let staff: Staff = Staff::new(i.to_string(), Client::new());
            println!("------------------------- [SPAWN-STAFF-ID] {} -------------------------", staff.get_table_id());
            staff.work();
        });
    }
    pool.join();
}