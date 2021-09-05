mod order_type;
mod settings;
mod staff;
mod tablet;

use settings::Settings;
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
            let staff: Staff = Staff::new(i.to_string());
            println!(
                "------------------------- [SPAWN-STAFF-ID] {} -------------------------",
                staff.get_table_id()
            );
            staff.work();
        });
    }
    pool.join();
}
