use crate::order_type::ItemPair;
use async_trait::async_trait;
use reqwest::Client;
use reqwest::Error;

#[async_trait]
pub trait Tablet {
    fn get_table_id(&self) -> String;
    fn get_inst(&self) -> Client;
    fn work(self);
    async fn place_order(&self, table_id: String, items: Vec<ItemPair>) -> Result<(), Error>;
    async fn delete_order(&self, table_id: String, item: String) -> Result<(), Error>;
    async fn update_order(&self, table_id: String, items: Vec<ItemPair>) -> Result<(), Error>;
    async fn status_order_all(&self, table_id: String) -> Result<(), Error>;
    async fn status_order_item(&self, table_id: String, item: String) -> Result<(), Error>;
}