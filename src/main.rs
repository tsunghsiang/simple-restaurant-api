use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ItemPair {
    name: String,
    amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaceOrder {
    table_id: String,
    items: Vec<ItemPair>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteOrder {
    table_id: String,
    item: String,
}

#[derive(Debug, Deserialize)]
struct UpdateOrder {
    table_id: String,
    items: Vec<ItemPair>,
}

#[async_std::main]
async fn main() -> tide::Result<()>{
    tide::log::start();
    let mut server = tide::new();
    
    /* simple api processing here */
    server.at("/api/status/order/:tableid").get(query_by_tableid);
    server.at("/api/status/order/:tableid/:item").get(query_by_tableid_and_item);
    server.at("/api/place/order").post(add_by_tableid_and_item);
    server.at("/api/delete/order").delete(remove_by_tableid_and_item);
    server.at("/api/update/order").put(update_by_tableid_and_item);

    server.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn query_by_tableid(req: tide::Request<()>) -> tide::Result {
    let mut collection = req.url().as_str().split('/');
    let table_id = collection.nth_back(0).unwrap();
    let res = format!("[QUERY] table id: {}", table_id);
    Ok(res.into())
}

async fn query_by_tableid_and_item(req: tide::Request<()>) -> tide::Result {
    let mut collection = req.url().as_str().split('/');
    let item = collection.nth_back(0).unwrap();
    let table_id = collection.nth_back(1).unwrap();
    let res = format!("[QUERY] table id: {}, item: {}", table_id, item);
    Ok(res.into())
}

async fn add_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let PlaceOrder { table_id, items } = req.body_json().await?;
    let res = format!("[PLACE] table id: {}, Order Nums: {}", table_id, items.len());
    Ok(res.into())
}

async fn remove_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let DeleteOrder { table_id, item } = req.body_json().await?;
    let res = format!("[DELETE] table id: {}, item: {}", table_id, item);
    Ok(res.into())
}

async fn update_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let UpdateOrder { table_id, items } = req.body_json().await?;
    let res = format!("[Update] table id: {}, Update Nums: {}", table_id, items.len());
    Ok(res.into())
}