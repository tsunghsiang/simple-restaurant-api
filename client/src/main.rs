mod tablet;
mod staff;
mod order_type;

use tablet::Tablet;
use staff::Staff;
use reqwest::Client;
use std::thread;
use std::time::Duration;

fn main() {
    // Start staffs to serve customers
    for i in (1..11) {
        let staff: Staff = Staff::new(i.to_string(), Client::new());
        staff.work();
        thread::sleep(Duration::from_millis(100));    
    }
    loop{}
}