use chrono::{DateTime, Utc};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use postgres_types;
use serde::{Deserialize, Serialize};
use std::string::ToString;
use strum_macros::Display;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum ReqType {
    Place = 0,
    Delete = 1,
    Update = 2,
    StatusAll = 3,  // show all items for a specific table
    StatusItem = 4, // show specific item for a specified table
}

#[derive(Display, Debug, ToSql, FromSql)]
#[postgres(name = "tablestatus")]
pub enum TableStatus {
    #[postgres(name = "Open")]
    Open,
    #[postgres(name = "Close")]
    Close,
}

#[derive(Display, Debug, ToSql, FromSql)]
#[postgres(name = "itemstatus")]
pub enum ItemStatus {
    #[postgres(name = "New")]
    New,
    #[postgres(name = "Process")]
    Process,
    #[postgres(name = "Done")]
    Done,
    #[postgres(name = "Deleted")]
    Deleted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemPair {
    pub name: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrder {
    pub created_at: DateTime<Utc>,
    pub table_id: String,
    pub items: Vec<ItemPair>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrder {
    pub deleted_at: DateTime<Utc>,
    pub table_id: String,
    pub item: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrder {
    pub updated_at: DateTime<Utc>,
    pub table_id: String,
    pub items: Vec<ItemPair>,
}

impl PlaceOrder {
    pub fn disp(&self) -> String {
        let mut res = "".to_owned();
        res.push_str("{ created_at: ");
        res.push_str(&self.created_at.to_string());
        res.push_str(", table_id: ");
        res.push_str(&self.table_id);
        res.push_str(", items: [ ");
        for i in 0..self.items.len() {
            res.push_str(&self.items[i].name);
            res.push_str(":");
            res.push_str(&self.items[i].amount.to_string());
            if i != self.items.len() - 1 {
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
        res.push_str("{ updated_at: ");
        res.push_str(&self.updated_at.to_string());
        res.push_str(", table_id: ");
        res.push_str(&self.table_id);
        res.push_str(", items: [ ");
        for i in 0..self.items.len() {
            res.push_str(&self.items[i].name);
            res.push_str(":");
            res.push_str(&self.items[i].amount.to_string());
            if i != self.items.len() - 1 {
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
        res.push_str("{ deleted_at: ");
        res.push_str(&self.deleted_at.to_string());
        res.push_str(", table_id:");
        res.push_str(&self.table_id);
        res.push_str(", items: ");
        res.push_str(&self.item);
        res.push_str(" }");
        res
    }
}
