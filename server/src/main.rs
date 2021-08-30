mod order_type;

use order_type::PlaceOrder;
use order_type::DeleteOrder;
use order_type::UpdateOrder;

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
    let order: PlaceOrder = req.body_json().await?;
    Ok(order.disp().into())
}

async fn remove_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let order: DeleteOrder = req.body_json().await?;
    Ok(order.disp().into())
}

async fn update_by_tableid_and_item(mut req: tide::Request<()>) -> tide::Result {
    let order: UpdateOrder = req.body_json().await?;
    Ok(order.disp().into())
}