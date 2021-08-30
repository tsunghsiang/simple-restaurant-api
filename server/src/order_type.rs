use serde::{Deserialize, Serialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum ReqType {
    Place = 0,
    Delete = 1,
    Update = 2,
    StatusAll = 3,     // show all items for a specific table
    StatusItem = 4,    // show specific item for a specified table
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemPair {
    pub name: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrder {
    pub timestamp: u64,
    pub table_id: String,
    pub items: Vec<ItemPair>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrder {
    pub timestamp: u64,
    pub table_id: String,
    pub item: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrder {
    pub timestamp: u64,
    pub table_id: String,
    pub items: Vec<ItemPair>,
}

impl PlaceOrder {
    pub fn disp(&self) -> String {
        let mut res = "".to_owned();
        res.push_str("{ timestamp:");
        res.push_str(&self.timestamp.to_string());
        res.push_str(", table_id:");
        res.push_str(&self.table_id);
        res.push_str(", items:[ ");
        for i in 0..self.items.len() {
            res.push_str(&self.items[i].name);
            res.push_str(":");
            res.push_str(&self.items[i].amount.to_string());
            if i != self.items.len()-1 {
                res.push_str(", ");
            }
        }
        res.push_str(" ]");
        res.push_str(" }");
        res
    }
}

impl UpdateOrder {
    pub fn disp(&self) -> String {
        let mut res = "".to_owned();
        res.push_str("{ timestamp:");
        res.push_str(&self.timestamp.to_string());
        res.push_str(", table_id:");
        res.push_str(&self.table_id);
        res.push_str(", items:[ ");
        for i in 0..self.items.len() {
            res.push_str(&self.items[i].name);
            res.push_str(":");
            res.push_str(&self.items[i].amount.to_string());
            if i != self.items.len()-1 {
                res.push_str(", ");
            }
        }
        res.push_str(" ]");
        res.push_str(" }");
        res
    }
}

impl DeleteOrder {
    pub fn disp(&self) -> String {
        let mut res = "".to_owned();
        res.push_str("{ timestamp:");
        res.push_str(&self.timestamp.to_string());
        res.push_str(", table_id:");
        res.push_str(&self.table_id);
        res.push_str(", items: ");
        res.push_str(&self.item);
        res.push_str(" }");
        res       
    }
}