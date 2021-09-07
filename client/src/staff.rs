use crate::order_type::{DeleteOrder, ItemPair, PlaceOrder, ReqType, UpdateOrder};
use crate::settings::{Auth, Settings};
use crate::tablet::Tablet;

use async_trait::async_trait;
use rand::Rng;
use reqwest::Client;
use reqwest::Error;
use std::convert::TryFrom;
use std::thread;
use std::time::Duration;
use std::vec::Vec;
use tokio::runtime::Runtime;

pub struct Staff {
    table_id: String,
    auth: Auth,
}

impl Staff {
    pub fn new(table_id: String, auth: Auth) -> Staff {
        Staff {
            table_id: table_id,
            auth: auth,
        }
    }
}

#[async_trait]
impl Tablet for Staff {
    fn get_table_id(&self) -> String {
        self.table_id.clone()
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
                        self.place_order(self.get_table_id(), generate_items())
                            .await
                    });
                }
                Ok(ReqType::Delete) => {
                    runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.delete_order(self.get_table_id(), get_item(val)).await
                    });
                }
                Ok(ReqType::Update) => {
                    runtime.block_on(async {
                        self.update_order(self.get_table_id(), generate_items())
                            .await
                    });
                }
                Ok(ReqType::StatusAll) => {
                    runtime.block_on(async { self.status_order_all(self.get_table_id()).await });
                }
                Ok(ReqType::StatusItem) => {
                    runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.status_order_item(self.get_table_id(), get_item(val))
                            .await
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

        let mut url: String = "".to_string();
        url.push_str(&Settings::get_base_url());
        url.push_str(&Settings::get_place_order_api());

        println!("[STAFF-{}][PLACE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .post(url)
            .header("X-Auth-Username", self.auth.get_username())
            .header("X-Auth-Password", self.auth.get_password())
            .json(&order)
            .send()
            .await?;
        let msg = resp.text().await?;
        println!("[STAFF-{}][PLACE][RESPONSE] {:?}", id, msg);
        Ok(())
    }

    async fn delete_order(&self, table_id: String, item: String) -> Result<(), Error> {
        let id = table_id.clone();
        let order: DeleteOrder = DeleteOrder {
            timestamp: timestamp(),
            table_id: table_id,
            item: item,
        };

        let mut url: String = "".to_string();
        url.push_str(&Settings::get_base_url());
        url.push_str(&Settings::get_delete_order_api());

        println!("[STAFF-{}][DELETE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .delete(url)
            .header("X-Auth-Username", self.auth.get_username())
            .header("X-Auth-Password", self.auth.get_password())
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

        let mut url: String = "".to_string();
        url.push_str(&Settings::get_base_url());
        url.push_str(&Settings::get_update_order_api());

        println!("[STAFF-{}][UPDATE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .patch(url)
            .header("X-Auth-Username", self.auth.get_username())
            .header("X-Auth-Password", self.auth.get_password())
            .json(&order)
            .send()
            .await?;
        let msg = resp.text().await?.to_string();
        println!("[STAFF-{}][UPDATE][RESPONSE] {:?}", id, msg);
        Ok(())
    }

    async fn status_order_all(&self, table_id: String) -> Result<(), Error> {
        let id = table_id.clone();
        let mut url: String = "".to_string();
        url.push_str(&Settings::get_base_url());
        url.push_str(&Settings::get_status_order_api());
        url.push_str("/");
        url.push_str(&table_id);

        println!("[STAFF-{}][STATUS_ALL][REQUEST] SENT! TABLE: {}", id, id);
        let executor = Client::new();
        let resp = executor.get(url).send().await?;
        let msg = resp.text().await?.to_string();
        println!("[STAFF-{}][STATUS_ALL][RESPONSE] {:?}", id, msg);
        Ok(())
    }

    async fn status_order_item(&self, table_id: String, item: String) -> Result<(), Error> {
        let mut url: String = "".to_string();
        url.push_str(&Settings::get_base_url());
        url.push_str(&Settings::get_status_order_api());
        url.push_str("/");
        url.push_str(&table_id);
        url.push_str("/");
        url.push_str(&item);

        println!(
            "[STAFF-{}][STATUS_ITEM][REQUEST] SENT! TABLE: {} CHECK ITEM: {}",
            table_id, table_id, item
        );
        let executor = Client::new();
        let resp = executor.get(url).send().await?;
        let msg = resp.text().await?.to_string();
        println!("[STAFF-{}][STATUS_ITEM][RESPONSE] {:?}", table_id, msg);
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
        _ => "A",
    })
    .to_string()
}

fn generate_items() -> Vec<ItemPair> {
    let mut items = Vec::<ItemPair>::new();
    let mut rng = rand::thread_rng();
    let cnt = rng.gen_range(2..27);
    for i in 0..(cnt - 1) {
        items.push(ItemPair {
            name: get_item(i),
            amount: rng.gen_range(1..10),
        });
    }
    items
}

fn timestamp() -> i64 {
    let timespec = time::get_time();
    let mills: i64 = (timespec.sec as i64 * 1000) + (timespec.nsec as i64 / 1000 / 1000);
    mills
}
