use crate::order_type::{ PlaceOrder, UpdateOrder, DeleteOrder };
use postgres::Error;

pub trait DB {
    fn init(&self) -> Result<(), Error>;
    //fn place(&self, order: PlaceOrder) -> Result<(), Error>;
    //fn place_order_proc(&self, order: PlaceOrder) -> Result<(), Error>;
    //fn update(&self, order: UpdateOrder) -> Result<String, Error>;
    //fn delete(&self, order: DeleteOrder) -> Result<String, Error>;
    fn query_by_tableid(&self, table_id: String) -> Result<String, Error>;
    fn query_by_tableid_and_item(&self, table_id: String, item: String) -> Result<String, Error>;
}