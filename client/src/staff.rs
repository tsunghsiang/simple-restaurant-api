use crate::tablet::Tablet;
use crate::order_type::{ReqType, ItemPair, PlaceOrder, DeleteOrder, UpdateOrder};
use reqwest::Client;
use reqwest::Error;
use std::thread;
use std::vec::Vec;
use std::time::Duration;
use rand::Rng;
use async_trait::async_trait;
use tokio::runtime::Runtime;
use std::convert::TryFrom;

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
            // random number generator
            let mut rng = rand::thread_rng();
            let duration = rng.gen_range(1..2);
            let req_type = rng.gen_range(0..5);

            match ReqType::try_from(req_type) {
                Ok(ReqType::Place) => {
                    runtime.block_on(async {
                        self.place_order(self.get_table_id(), generate_items()).await
                    });
                },
                Ok(ReqType::Delete) => {
                    runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.delete_order(self.get_table_id(), get_item(val)).await
                    });
                },
                Ok(ReqType::Update) => {
                    runtime.block_on(async {
                        self.update_order(self.get_table_id(), generate_items()).await
                    });
                },
                Ok(ReqType::StatusAll) => {
                    runtime.block_on(async {
                        self.status_order_all(self.get_table_id()).await
                    });
                },
                Ok(ReqType::StatusItem) => {
                    runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.status_order_item(self.get_table_id(), get_item(val)).await
                    });                   
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(duration * 1000));
        }
    }

    async fn place_order(&self, table_id: String, items: Vec<ItemPair>) -> Result<(), Error> {
        let id = table_id.clone();
        let order: PlaceOrder = PlaceOrder {
            timestamp: timestamp(),
            table_id: table_id,
            items: items,
        };
        println!("[STAFF-{}][PLACE][REQUEST] {}", id, order.disp()); 
        let executor = Client::new();       
        let resp = executor.post("http://127.0.0.1:8080/api/place/order")
                           .json(&order)
                           .send()
                           .await?;
        let msg = resp.text().await?;
        println!("[STAFF-{}][PLACE][RESPONSE] {:?}", id, msg); 
        Ok(())
    }

    async fn delete_order(&self, table_id: String, item: String) -> Result<(), Error>{
        let id = table_id.clone();
        let order: DeleteOrder = DeleteOrder {
            timestamp: timestamp(),
            table_id: table_id,
            item: item,
        };
        println!("[STAFF-{}][DELETE][REQUEST] {}", id, order.disp()); 
        let executor = Client::new();       
        let resp = executor.delete("http://127.0.0.1:8080/api/delete/order")
                           .json(&order)
                           .send()
                           .await?;
        let msg = resp.text().await?;
        println!("[STAFF-{}][DELETE][RESPONSE] {:?}", id, msg); 
        Ok(())
    }

    async fn update_order(&self, table_id: String, items: Vec<ItemPair>) -> Result<(), Error> {
        let id = table_id.clone();
        let order: UpdateOrder = UpdateOrder {
            timestamp: timestamp(),
            table_id: table_id,
            items: items,
        };
        println!("[STAFF-{}][UPDATE][REQUEST] {}", id, order.disp()); 
        let executor = Client::new();       
        let resp = executor.put("http://127.0.0.1:8080/api/update/order")
                           .json(&order)
                           .send()
                           .await?;
        let msg = resp.text().await?.to_string();
        println!("[STAFF-{}][UPDATE][RESPONSE] {:?}", id, msg); 
        Ok(())
    }

    async fn status_order_all(&self, table_id: String) -> Result<(), Error> {
        let id = table_id.clone();
        let url: String = format!("http://127.0.0.1:8080/api/status/order/{}", table_id);
        println!("[STAFF-{}][STATUS_ALL][REQUEST] SENT! TABLE: {}", id, id);
        let resp = reqwest::get(url).await?
                                    .text()
                                    .await?; 
        println!("[STAFF-{}][STATUS_ALL][RESPONSE] {:?}", id, resp);
        Ok(()) 
    }

    async fn status_order_item(&self, table_id: String, item: String) -> Result<(), Error> {
        let (id, term) = (table_id.clone(), item.clone());
        let url: String = format!("http://127.0.0.1:8080/api/status/order/{}/{}", table_id, item);
        println!("[STAFF-{}][STATUS_ITEM][REQUEST] SENT! TABLE: {} CHECK ITEM: {}", id, id, term);
        let resp = reqwest::get(url).await?
                                    .text()
                                    .await?;
        println!("[STAFF-{}][STATUS_ITEM][RESPONSE] {:?}", id, resp);
        Ok(())
    }

}

fn get_item(val: i8) -> String {
    (match val {
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

fn generate_items() -> Vec<ItemPair> {
    let mut items = Vec::<ItemPair>::new();
    let mut rng = rand::thread_rng();
    let cnt = rng.gen_range(2..27);
    for i in 0..(cnt-1) {
        items.push(ItemPair{
            name: get_item(i),
            amount: rng.gen_range(1..10),
        });
    }
    items
}

fn timestamp() -> u64 {
    let timespec = time::get_time();
    let mills: u64 = (timespec.sec as u64 * 1000) + (timespec.nsec as u64 / 1000 / 1000);
    mills
}