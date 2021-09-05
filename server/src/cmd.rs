use crate::db::DB;
use crate::order_type::{ ItemPair, PlaceOrder, UpdateOrder, DeleteOrder };
use std::time::Duration;
use std::thread;
use postgres::{Client, NoTls, Error};
use rand::Rng;
use futures::executor::block_on;

pub struct Dbio {
    name: String,
}

impl Dbio {
    pub fn new() -> Dbio {
        Dbio {
            name: "postgresql://postgres:nctusrs0915904265@localhost:5432/restaurant".to_string()
        }
    }

    pub fn get_db_path(&self) -> &str {
        &self.name
    }
}

impl DB for Dbio {
    fn init(&self) -> Result<(), Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls)?;
        
        client.batch_execute("
            CREATE TABLE IF NOT EXISTS tablet (
                id              SERIAL PRIMARY KEY,
                timestamp       BIGINT NOT NULL,
                table_id        VARCHAR NOT NULL,
                table_status    VARCHAR NOT NULL
            )
        ")?;

        client.batch_execute("
            CREATE TABLE IF NOT EXISTS items  (
                id          SERIAL PRIMARY KEY,
                timestamp   BIGINT NOT NULL,
                table_id    VARCHAR NOT NULL,
                item        VARCHAR NOT NULL,
                amount      INTEGER,
                item_status VARCHAR NOT NULL,
                cook_time   INTEGER
            )
        ")?;

        Ok(())        
    }

    fn place(&self, order: PlaceOrder) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls)?;
        let mut res: String = "".to_string();

        match client.query_one("SELECT timestamp,
                                       table_id,
                                       table_status
                                       FROM tablet WHERE table_id = $1 AND timestamp = (SELECT MAX(timestamp)
                                                                                        FROM tablet
                                                                                        WHERE table_id = $2) FOR UPDATE", &[&order.table_id, &order.table_id]) {
            Ok(row) => {
                let table_id: String = row.get(1);
                let table_status: String = row.get(2);
                
                if table_status.eq("todo") {
                    res = format!("Duplicated! There has been an order of table_id: {}, waiting in the queue", table_id);
                } else if table_status.eq("doing") {
                    res = format!("Duplicated! There has been an order of table_id: {}, cooking in the kitchen", table_id);
                } else if table_status.eq("done") {
                    res = format!("New Order! timestamp: {}, table_id: {}", order.timestamp, order.table_id);
                    // insert new order into table 'tablet'
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES ($1, $2, 'todo')", &[&order.timestamp, &order.table_id])?;
                    // insert new items into table 'items'
                    let mut rng = rand::thread_rng();
                
                    for elem in order.items {
                        let (ts, table_id, item, amount, status, cook_time) = (order.timestamp, order.table_id.to_string(), elem.name, elem.amount, "'todo'".to_string(), rng.gen_range(5..16));    
                        client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status, cook_time) VALUES ($1, $2, $3, $4, $5, $6)",
                                        &[&ts, &table_id, &item, &amount, &status, &cook_time])?;
                        // spawn a task handling item preparation
                        thread::spawn(move || block_on(cook_order_item(ts, &table_id.to_string(), &item.to_string(), cook_time)));
                    }
                }
            },
            Err(_err) => {
                res = format!("New Order! timestamp: {}, table_id: {}", order.timestamp, order.table_id);
                // insert new order into table 'tablet'
                client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES ($1, $2, 'todo')", &[&order.timestamp, &order.table_id])?;

                // insert new items into table 'items'
                let mut rng = rand::thread_rng();
            
                for elem in order.items {
                    let (ts, table_id, item, amount, status, cook_time) = (order.timestamp, order.table_id.to_string(), elem.name, elem.amount, "'todo'".to_string(), rng.gen_range(5..16));    
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status, cook_time) VALUES ($1, $2, $3, $4, $5, $6)",
                                    &[&ts, &table_id, &item, &amount, &status, &cook_time])?;
                    // spawn a task handling item preparation
                    thread::spawn(move || block_on(cook_order_item(ts, &table_id.to_string(), &item.to_string(), cook_time)));
                }
            }
        }
        
        Ok(res.into())
    }

    fn update(&self, order: UpdateOrder) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        let mut res: String = "".to_string();

        match client.query_one("SELECT timestamp, table_id, table_status
                                FROM tablet
                                WHERE table_id = $1 AND timestamp = (SELECT MAX(timestamp)
                                                                     FROM tablet
                                                                     WHERE table_id = $1) FOR UPDATE", &[&order.table_id]) {
            Ok(row) => {
                let timestamp: i64 = row.get("timestamp");
                let table_id: String = row.get("table_id");
                let table_status: String = row.get("table_status");
                if table_status.eq("done") {
                    res = format!("Update Order Failed! table_id: {} was done. Please launch a new order", table_id);
                } else if table_status.eq("todo") || table_status.eq("doing") {                   
                    for elem in order.items {
                        let id = table_id.clone();
                        thread::spawn(move || update_order_item(timestamp, id, elem));
                    }
                    res = format!("Update Order Successed! table_id: {}", table_id);
                }
            },
            Err(_err) => {
                res = format!("Update Order Failed! Non-existent table_id: {}", order.table_id);
            }
        };

        Ok(res.into())
    }

    fn delete(&self, order: DeleteOrder) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        let mut res: String = "".to_string();
        match client.query_one("SELECT timestamp, table_id, item, amount, item_status
                                FROM items
                                WHERE table_id = $1 AND item = $2 and timestamp = ( SELECT MAX(timestamp)
                                                                                    FROM tablet
                                                                                    WHERE table_id = $1 ) FOR UPDATE", &[&order.table_id, &order.item]){
            Ok(row) => {
                let ts: i64 = row.get("timestamp");
                let table_id: String = row.get("table_id");
                let item: String = row.get("item");
                let item_status: String = row.get("item_status");

                if item_status.eq("doing") {
                    res = format!("Delete Order Failed! The item: {} of table_id: {} is cooking", item, table_id);
                } else if item_status.eq("done") {
                    res = format!("Delete Order Failed! The item: {} of table_id: {} was done", item, table_id);
                } else {
                    match client.execute("DELETE FROM items 
                                          WHERE table_id = $1 AND item = $2 AND item_status = $3 and timestamp = (SELECT MAX(timestamp)
                                                                                                                  FROM tablet
                                                                                                                  WHERE table_id = $1);", &[&order.table_id, &order.item, &"todo".to_string()]) {
                        Ok(_n) => {
                            res = format!("Delete Order Successed! item: {} of table_id: {} deleted", order.item, order.table_id);
                            update_table_status(client, table_id, ts)?;
                        },
                        Err(_err) => {
                            res = format!("Delete Order Failed! No item: {} of table_id: {}", order.item, order.table_id);                           
                        }
                    }
                }
            },
            Err(_err) => {
                res = format!("Delete Order Failed! No item: {} of table_id: {}", order.item, order.table_id);
            }
        };

        Ok(res.into())
    }

    fn query_by_tableid(&self, table_id: String) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        
        let mut res = "".to_owned();
        let ts: i64 = timestamp();
        let mut empty: bool = true;
        res.push_str("{ timestamp:");
        res.push_str(&ts.to_string());
        res.push_str(", table_id:");
        res.push_str(&table_id);
        res.push_str(", items:[ ");
        
        for row in client.query("SELECT item, amount, item_status
                                 FROM items
                                 WHERE table_id = $1 AND timestamp = (SELECT MAX(timestamp)
                                                                      FROM tablet
                                                                      WHERE table_id = $2) FOR UPDATE", &[&table_id, &table_id]).unwrap() {
            empty = false;
            let item: String = row.get(0);
            let amount: i32 = row.get(1);
            let item_status: String = row.get(2);

            res.push_str("{ item: ");
            res.push_str(&item);
            res.push_str(", amount: ");
            res.push_str(&amount.to_string());
            res.push_str(", item_status: ");
            res.push_str(&item_status);
            res.push_str(" },");
        }

        if empty {
            res = format!("No Order of table id: {}", table_id);
        } else {
            res.push_str("]");
            res.push_str(" }");
        }

        Ok(res.into())
    }

    fn query_by_tableid_and_item(&self, table_id: String, item: String) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        
        let mut res = "".to_owned();
        let ts: i64 = timestamp();
        let mut empty: bool = true;
        res.push_str("{ timestamp:");
        res.push_str(&ts.to_string());
        
        for row in client.query("SELECT table_id, item, amount, item_status
                                 FROM items
                                 WHERE table_id = $1 AND item = $2 AND timestamp = (SELECT MAX(timestamp)
                                                                                    FROM tablet
                                                                                    WHERE table_id = $1) FOR UPDATE", &[&table_id, &item]).unwrap() {
            empty = false;
            let table_id: String = row.get(0);
            let item: String = row.get(1);
            let amount: i32 = row.get(2);
            let item_status: String = row.get(3);

            res.push_str(", table_id: ");
            res.push_str(&table_id);
            res.push_str(", item: ");
            res.push_str(&item);
            res.push_str(", amount: ");
            res.push_str(&amount.to_string());
            res.push_str(", item_status: ");
            res.push_str(&item_status);
        }

        if empty {
            res = format!("No Order of table id: {}", table_id);
        } else {
            res.push_str(" }");
        }

        Ok(res.into())
    }

    fn check_table_status(&self) -> Result<bool, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        let mut is_empty: bool = true;

        for row in client.query("SELECT * FROM tablet WHERE table_status NOT IN ('done')", &[])? {
            is_empty = false;
            let table_id: String = row.get("table_id");
            let table_status: String = row.get("table_status");
            if table_status.eq("todo") {
                println!("[CHECK_TABLE_STATUS] table_id: {} is still waiting, status: {}", table_id, table_status);
            } else {
                println!("[CHECK_TABLE_STATUS] table_id: {} is still cooking, status: {}", table_id, table_status);
            }
        }

        Ok(is_empty)
    }
}

fn timestamp() -> i64 {
    let timespec = time::get_time();
    let mills: i64 = (timespec.sec as i64 * 1000) + (timespec.nsec as i64 / 1000 / 1000);
    mills
}

fn update_table_status(mut client: Client, table_id: String, timestamp: i64) -> Result<(), Error> {
    let mut empty: bool = true;
    let (mut todo, mut doing, mut done) = (false, false, false);
    
    for row in client.query("SELECT timestamp, table_id, item, amount, item_status
                             FROM items
                             WHERE table_id = $1 AND timestamp = $2 FOR UPDATE", &[&table_id, &timestamp])? {
        empty = false;
        let status: String = row.get(4);
        
        if status.eq("todo") {
            todo = true;
        } else if status.eq("doing") {
            doing = true;
        } else {
            done = true;
        }
    }

    if empty {
        client.execute("UPDATE tablet
                        SET table_status = 'done'
                        WHERE table_id = $1 and timestamp = $2", &[&table_id, &timestamp])?;
        ()
    }

    if doing {
        client.execute("UPDATE tablet
                        SET table_status = 'doing'
                        WHERE table_id = $1 and timestamp = $2", &[&table_id, &timestamp])?;
    } else {
        if todo {
            client.execute("UPDATE tablet
                            SET table_status = 'todo'
                            WHERE table_id = $1 and timestamp = $2", &[&table_id, &timestamp])?;
        } else if done {
            client.execute("UPDATE tablet
                            SET table_status = 'done'
                            WHERE table_id = $1 and timestamp = $2", &[&table_id, &timestamp])?;
        }
    }

    Ok(())
}

async fn update_order_item(timestamp: i64, table_id: String, elem: ItemPair) {
    let command: Dbio = Dbio::new();
    let mut client = Client::connect(command.get_db_path(), NoTls).unwrap();
    let mut res: String = "".to_string();       
    
    match client.query_one("SELECT timestamp, table_id, item, amount, item_status
                            FROM items
                            WHERE table_id = $1 AND item = $2 AND timestamp = $3 FOR UPDATE", &[&table_id, &elem.name, &timestamp]) {
        Ok(row) => {
            let timestamp: i64 = row.get("timestamp");
            let item_status: String = row.get("item_status");
            let amount: i32 = row.get("amount");

            if item_status.eq("todo") {
                match client.execute("UPDATE items
                                      SET amount = $1
                                      WHERE table_id = $2 AND item = $3 AND item_status = 'todo' AND timestamp = $4", &[&elem.amount, &table_id, &elem.name, &timestamp]) {
                    Ok(_n) => res = format!("Update Item Successed! The item: {} table_id :{} amount: {} -> {}", elem.name, table_id, amount, elem.amount),
                    Err(e) => res = format!("Update Item Failed! The item: {} of table_id: {}, {}", elem.name, table_id, e)
                }               
            } else if item_status.eq("doing") {
                res = format!("Update Item Failed! The item: {} of table_id :{} is cooking", elem.name, table_id);               
            } else {
                res = format!("Update Item Failed! The item: {} of table_id: {} was done", elem.name, table_id);
            }
        },
        Err(_err) => {
            let mut rng = rand::thread_rng();
            let cook_time: i32 = rng.gen_range(5..16);
            res = format!("Update Item Successed! New item: {} added to the table_id: {}\n", elem.name, table_id);
            // spawn a task handling item preparation
            update_item_status(timestamp, table_id.to_string(), elem.name.to_string(), "doing".to_string());
            thread::sleep(Duration::from_millis((cook_time * 1000) as u64));
            update_item_status(timestamp, table_id.to_string(), elem.name.to_string(), "done".to_string());
        }
    }

    println!("{}", res);
}

fn update_item_status(timestamp: i64, table_id: String, item: String, to: String) {
    let command: Dbio = Dbio::new();
    let mut client = Client::connect(command.get_db_path(), NoTls).unwrap();
    
    match client.execute("UPDATE items
                          SET item_status = $1
                          WHERE timestamp = $2 AND table_id = $3 AND item = $4", &[&to, &timestamp, &table_id, &item]) {
        // Ok(_n) => println!("[UPDATE_ITEM_STATUS] table_id: {}, item: {}, status: {}", table_id, item, to),
        // Err(err) => println!("[UPDATE_ITEM_STATUS] Cook Error: {}", err)
        Ok(_) => {},
        Err(err) => println!("[UPDATE_ITEM_STATUS] Cook Error: {}", err)
    };
    
    match update_table_status(client, table_id, timestamp) {
        Ok(()) => {},
        Err(_) => {}
    };
}

async fn cook_order_item(timestamp: i64, table_id: &str, item: &str, cook_time: i32) {
    update_item_status(timestamp, table_id.to_string(), item.to_string(), "doing".to_string());
    thread::sleep(Duration::from_millis((cook_time * 1000) as u64));
    update_item_status(timestamp, table_id.to_string(), item.to_string(), "done".to_string());
}