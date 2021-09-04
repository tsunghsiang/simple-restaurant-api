mod cmd;
mod db;
mod order_type;

use cmd::Dbio;
use db::DB;
use order_type::DeleteOrder;
use order_type::PlaceOrder;
use order_type::UpdateOrder;

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let mut server = tide::new();
    let command: Dbio = Dbio::new();
    /* Check DB status first */
    match command.init() {
        Ok(()) => println!("[SERVER] DB status OK"),
        Err(err) => {
            println!("[SERVER] DB Error: {}", err);
            ()
        }
    }

    /* simple api processing here */
    server
        .at("/api/status/order/:tableid")
        .get(query_by_tableid);
    server
        .at("/api/status/order/:tableid/:item")
        .get(query_by_tableid_and_item);
    server.at("/api/place/order").post(add_by_tableid_and_item);
    server
        .at("/api/delete/order")
        .delete(remove_by_tableid_and_item);
    server
        .at("/api/update/order")
        .put(update_by_tableid_and_item);
    server.listen("127.0.0.1:8080").await?;

    Ok(())
}

async fn query_by_tableid(req: tide::Request<()>) -> tide::Result {
    let mut collection = req.url().as_str().split('/');
    let mut res: String = "".to_string();
    let command: Dbio = Dbio::new();
    let table_id = collection.nth_back(0).unwrap();

    match command.query_by_tableid(table_id.to_string()) {
        Ok(result) => res = result,
        _ => {}
    };

    Ok(res.into())
}

async fn query_by_tableid_and_item(req: tide::Request<()>) -> tide::Result {
    let mut collection = req.url().as_str().split('/');
    let mut res: String = "".to_string();
    let command: Dbio = Dbio::new();
    let item = collection.nth_back(0).unwrap();
    let table_id = collection.nth_back(0).unwrap();

    match command.query_by_tableid_and_item(table_id.to_string(), item.to_string()) {
        Ok(result) => res = result,
        _ => {}
    };

    Ok(res.into())
}

async fn add_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let order: PlaceOrder = req.body_json().await?;
    let command: Dbio = Dbio::new();
    let mut res: String = "".to_string();
    match command.place(order) {
        Ok(result) => res = result,
        _ => {}
    };
    Ok(res.into())
}

async fn remove_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let order: DeleteOrder = req.body_json().await?;
    let command: Dbio = Dbio::new();
    let mut res: String = "".to_string();
    match command.delete(order) {
        Ok(result) => res = result,
        _ => {}
    };
    Ok(res.into())
}

async fn update_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let order: UpdateOrder = req.body_json().await?;
    let command: Dbio = Dbio::new();
    let mut res: String = "".to_string();
    match command.update(order) {
        Ok(result) => res = result,
        _ => {}
    };
    Ok(res.into())
}
