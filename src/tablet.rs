use crate::order_type::ItemPair;

pub trait Tablet {
    fn get_table_id(&self) -> String;
    fn work(&self);
    fn place_order(&self, table_id: String, items: Vec<ItemPair>);
    fn delete_order(&self, table_id: String, item: String);
    fn update_order(&self, table_id: String, items: Vec<ItemPair>);
    fn status_order_all(&self, table_id: String);
    fn status_order_item(&self, table_id: String, item: String);
}