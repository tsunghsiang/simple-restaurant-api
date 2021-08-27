use crate::tablet::Tablet;
use crate::order_type::ItemPair;
use reqwest::Client;
use reqwest::Error;
use std::thread;
use std::time::Duration;
use rand::Rng;
use async_trait::async_trait;
use tokio::runtime::Runtime;
use threadpool::ThreadPool;

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

#[async_trait]
impl Tablet for Staff {
    
    fn get_table_id(&self) -> String {
        self.table_id.clone()
    }

    fn get_inst(&self) -> Client {
        self.inst.clone()
    }
    
    fn work(self) {
        let runtime = Runtime::new().unwrap();
        loop {
            let mut req_type: i8 = 0;
            // random number generator
            let mut rng = rand::thread_rng();
            let duration = rng.gen_range(1..2);
            req_type = rng.gen_range(1..5);

            match req_type {
                0 => {
                    //println!("[PLACE]");
                },
                1 => {
                    //println!("[DELETE]");
                },
                2 => {
                    //println!("[UPDATE]");
                },
                3 => {
                    runtime.block_on(async {
                        self.status_order_all(self.get_table_id()).await
                    });
                },
                4 => {
                    runtime.block_on(async {
                        self.status_order_item(self.get_table_id(), get_item()).await
                    });                   
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(duration * 1000));
        }
    }

    async fn place_order(&self, table_id: String, items: Vec<ItemPair>) {

    }

    async fn delete_order(&self, table_id: String, item: String) {

    }

    async fn update_order(&self, table_id: String, items: Vec<ItemPair>) {

    }

    async fn status_order_all(&self, table_id: String) -> Result<(), Error> {
        let id = table_id.clone();
        let url: String = format!("http://127.0.0.1:8080/api/status/order/{}", table_id);
        println!("[STAFF-{}][STATUS_ALL][REQUEST] SENT! TABLE: {}", id, id);
        let body = reqwest::get(url).await?
                                    .text()
                                    .await?;
        println!("[STAFF-{}][STATUS_ALL][RESPONSE] {:?}", id, body);
        Ok(()) 
    }

    async fn status_order_item(&self, table_id: String, item: String) -> Result<(), Error>{
        let (id, term) = (table_id.clone(), item.clone());
        let url: String = format!("http://127.0.0.1:8080/api/status/order/{}/{}", table_id, item);
        println!("[STAFF-{}][STATUS_ITEM][REQUEST] SENT! TABLE: {} CHECK ITEM: {}", id, id, term);
        let body = reqwest::get(url).await?
                                    .text()
                                    .await?;
        println!("[STAFF-{}][STATUS_ALL][RESPONSE] {:?}", id, body);
        Ok(()) 
    }

}

fn get_item() -> String {
    let item_num = rand::thread_rng().gen_range(0..26);
    (match item_num {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        5 => "F",
        6 => "G",
        7 => "H",
        8 => "I",
        9 => "J",
        10 => "K",
        11 => "L",
        12 => "M",
        13 => "N",
        14 => "O",
        15 => "P",
        16 => "Q",
        17 => "R",
        18 => "S",
        19 => "T",
        20 => "U",
        21 => "V",
        22 => "W",
        23 => "X",
        24 => "Y",
        25 => "Z",
        _ => "A"
    }).to_string()
}