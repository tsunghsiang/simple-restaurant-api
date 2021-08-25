use crate::tablet::Tablet;
use crate::order_type::ItemPair;
use reqwest::Client;
use std::thread;
use std::time::Duration;
use rand::Rng;

/*
enum ReqType {
    PLACE = 0,
    DELETE = 1,
    UPDATE = 2,
    STATUS_ALL = 3,     // show all items for a specific table
    STATUS_ITEM = 4,    // show specific item for a specified table
}*/

pub struct Staff {
    table_id: String,
    inst: Client,
}

impl Staff {
    pub fn new(table_id: String, inst: Client) -> Staff{
        Staff{
            table_id: table_id,
            inst: inst
        }
    }
}

impl Tablet for Staff {
    
    fn get_table_id(&self) -> String {
        self.table_id.clone()
    }
    
    fn work(&self) {
        let nm: String = format!("[STAFF-{}]", self.get_table_id());
        let builder: thread::Builder = thread::Builder::new().name(nm.clone());
        builder.spawn(move || {
            // case corresponds to request type
            let mut req_type: i8 = 0;
            // random number generator
            let mut rng = rand::thread_rng();
            loop {
                let duration = rng.gen_range(1..10);
                req_type = rng.gen_range(1..5);
                println!("{}[CASE] {} [DURATION] {} s", nm, req_type, duration);
                match req_type {
                    0 => {
                        println!("case 0");
                    },
                    1 => {
                        println!("case 1");
                    },
                    2 => {
                        println!("case 2");
                    },
                    3 => {
                        println!("case 3");
                    },
                    4 => {
                        println!("case 4");
                    }
                    _ => {}
                }
                thread::sleep(Duration::from_millis(duration * 1000));
            }
        });
        /*
            spawn a thread
            while(true) {
                random = random-number-generator
                switch(random) {
                    case -1:
                        place order
                        break;
                    case 0:
                        delete order
                        break;
                    case 1:
                        update order
                        break;
                    case 2:
                        status order with tableid
                        break;
                    case 3:
                        status order with tableid/item
                        break;
                    default:
                        break;
                }
            }
        */
    }

    fn place_order(&self, table_id: String, items: Vec<ItemPair>) {

    }

    fn delete_order(&self, table_id: String, item: String) {

    }

    fn update_order(&self, table_id: String, items: Vec<ItemPair>) {

    }

    fn status_order_all(&self, table_id: String) {

    }

    fn status_order_item(&self, table_id: String, item: String) {

    }

}