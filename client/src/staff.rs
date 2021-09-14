use crate::order_type::{DeleteOrder, ItemPair, PlaceOrder, ReqType, UpdateOrder};
use crate::settings::Settings;
use crate::tablet::Tablet;

use async_trait::async_trait;
use chrono::Utc;
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
    config: Settings,
}

impl Staff {
    pub fn new(table_id: String, config: Settings) -> Staff {
        Staff {
            table_id: table_id,
            config: config,
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
                    match runtime.block_on(async {
                        self.place_order(self.get_table_id(), generate_items())
                            .await
                    }) {
                        Ok(_) => {}
                        Err(e) => println!("[PLACE][ERROR] {}", e),
                    }
                }
                Ok(ReqType::Delete) => {
                    match runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.delete_order(self.get_table_id(), get_item(val)).await
                    }) {
                        Ok(_) => {}
                        Err(e) => println!("[DELETE][ERROR] {}", e),
                    }
                }
                Ok(ReqType::Update) => {
                    match runtime.block_on(async {
                        self.update_order(self.get_table_id(), generate_items())
                            .await
                    }) {
                        Ok(_) => {}
                        Err(e) => println!("[UPDATE][ERROR] {}", e),
                    }
                }
                Ok(ReqType::StatusAll) => {
                    match runtime
                        .block_on(async { self.status_order_all(self.get_table_id()).await })
                    {
                        Ok(_) => {}
                        Err(e) => println!("[STATUS_ALL][ERROR] {}", e),
                    }
                }
                Ok(ReqType::StatusItem) => {
                    match runtime.block_on(async {
                        let val: i8 = rng.gen_range(0..26);
                        self.status_order_item(self.get_table_id(), get_item(val))
                            .await
                    }) {
                        Ok(_) => {}
                        Err(e) => println!("[STATUS_ALL][ERROR] {}", e),
                    }
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(duration * 1000));
        }
    }

    async fn place_order(&self, table_id: String, items: Vec<ItemPair>) -> Result<(), Error> {
        let id = table_id.clone();
        let order: PlaceOrder = PlaceOrder {
            created_at: Utc::now(),
            table_id: table_id,
            items: items,
        };

        let mut url: String = "".to_string();
        url.push_str(&self.config.client.get_base_url());
        url.push_str(&self.config.api.get_place_order_api());

        println!("[STAFF-{}][PLACE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .post(url)
            .header("X-Auth-Username", self.config.auth.get_username())
            .header("X-Auth-Password", self.config.auth.get_password())
            .timeout(Duration::from_secs(self.config.client.get_timeout()))
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
            deleted_at: Utc::now(),
            table_id: table_id,
            item: item,
        };

        let mut url: String = "".to_string();
        url.push_str(&self.config.client.get_base_url());
        url.push_str(&self.config.api.get_delete_order_api());

        println!("[STAFF-{}][DELETE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .delete(url)
            .header("X-Auth-Username", self.config.auth.get_username())
            .header("X-Auth-Password", self.config.auth.get_password())
            .timeout(Duration::from_secs(self.config.client.get_timeout()))
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
            updated_at: Utc::now(),
            table_id: table_id,
            items: items,
        };

        let mut url: String = "".to_string();
        url.push_str(&self.config.client.get_base_url());
        url.push_str(&self.config.api.get_update_order_api());

        println!("[STAFF-{}][UPDATE][REQUEST] {}", id, order.disp());
        let executor = Client::new();
        let resp = executor
            .patch(url)
            .header("X-Auth-Username", self.config.auth.get_username())
            .header("X-Auth-Password", self.config.auth.get_password())
            .timeout(Duration::from_secs(self.config.client.get_timeout()))
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
        url.push_str(&self.config.client.get_base_url());
        url.push_str(&self.config.api.get_status_order_api());
        url.push_str("/");
        url.push_str(&table_id);

        println!("[STAFF-{}][STATUS_ALL][REQUEST] SENT! TABLE: {}", id, id);
        let executor = Client::new();
        let resp = executor
            .get(url)
            .timeout(Duration::from_secs(self.config.client.get_timeout()))
            .send()
            .await?;
        let msg = resp.text().await?.to_string();
        println!("[STAFF-{}][STATUS_ALL][RESPONSE] {:?}", id, msg);
        Ok(())
    }

    async fn status_order_item(&self, table_id: String, item: String) -> Result<(), Error> {
        let mut url: String = "".to_string();
        url.push_str(&self.config.client.get_base_url());
        url.push_str(&self.config.api.get_status_order_api());
        url.push_str("/");
        url.push_str(&table_id);
        url.push_str("/");
        url.push_str(&item);

        println!(
            "[STAFF-{}][STATUS_ITEM][REQUEST] SENT! TABLE: {} CHECK ITEM: {}",
            table_id, table_id, item
        );
        let executor = Client::new();
        let resp = executor
            .get(url)
            .timeout(Duration::from_secs(self.config.client.get_timeout()))
            .send()
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_staff_get_table_id_given_cofig_provided_when_instantiated_then_table_id_obtained() {
        let config: Settings = Settings::new();
        let staff: Staff = Staff::new("1".to_string(), config);
        assert_eq!("1", staff.get_table_id());
    }

    #[test]
    fn test_get_item_given_rand_num_provided_when_executed_then_an_alphabet_obtained() {
        assert_eq!("A", get_item(0));
        assert_eq!("B", get_item(1));
        assert_eq!("C", get_item(2));
        assert_eq!("D", get_item(3));
        assert_eq!("E", get_item(4));
        assert_eq!("F", get_item(5));
        assert_eq!("G", get_item(6));
        assert_eq!("H", get_item(7));
        assert_eq!("I", get_item(8));
        assert_eq!("J", get_item(9));
        assert_eq!("K", get_item(10));
        assert_eq!("L", get_item(11));
        assert_eq!("M", get_item(12));
        assert_eq!("N", get_item(13));
        assert_eq!("O", get_item(14));
        assert_eq!("P", get_item(15));
        assert_eq!("Q", get_item(16));
        assert_eq!("R", get_item(17));
        assert_eq!("S", get_item(18));
        assert_eq!("T", get_item(19));
        assert_eq!("U", get_item(20));
        assert_eq!("V", get_item(21));
        assert_eq!("W", get_item(22));
        assert_eq!("X", get_item(23));
        assert_eq!("Y", get_item(24));
        assert_eq!("Z", get_item(25));
        assert_eq!("A", get_item(26));
    }

    #[test]
    fn test_generate_items_given_at_least_one_rand_num_provided_when_invoked_then_vec_len_greater_than_zero(
    ) {
        let res: Vec<ItemPair> = generate_items();
        assert!(res.len() > 0);
    }
}
