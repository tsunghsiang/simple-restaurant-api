use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemPair {
    pub name: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrder {
    pub table_id: String,
    pub items: Vec<ItemPair>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrder {
    pub table_id: String,
    pub item: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrder {
    pub table_id: String,
    pub items: Vec<ItemPair>,
}