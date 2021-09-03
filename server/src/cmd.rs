use crate::db::DB;
use crate::order_type::{ ItemPair, PlaceOrder, UpdateOrder, DeleteOrder };
use std::vec::Vec;
use std::assert_eq;
use std::time::SystemTime;
use std::thread;
use postgres::{Client, NoTls, Error};
use postgres_types::ToSql;
use r2d2_postgres::{ PostgresConnectionManager, r2d2 };
use rand::Rng;

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
        } else {
            client.execute("UPDATE tablet
                            SET table_status = 'done'
                            WHERE table_id = $1 and timestamp = $2", &[&table_id, &timestamp])?;
        }
    }

    Ok(())
}

fn update_order_item(client: &mut Client, timestamp: i64, table_id: &String, elem: ItemPair) -> String {
    let mut res: String = "".to_string();       
    match client.query_one("SELECT timestamp, table_id, item, amount, item_status
                            FROM items
                            WHERE table_id = $1 AND item = $2 AND timestamp = (SELECT MAX(timestamp)
                                                                               FROM tablet
                                                                               WHERE table_id = $1) FOR UPDATE", &[&table_id, &elem.name]) {
        Ok(row) => {
            let item_status: String = row.get("item_status");
            let amount: i32 = row.get("amount");

            if item_status.eq("todo") {
                match client.execute("UPDATE items
                                      SET amount = $1
                                      WHERE table_id = $2 AND item = $3 AND item_status = 'todo' AND timestamp = (SELECT MAX(timestamp)
                                                                                                                  FROM tablet
                                                                                                                  WHERE table_id = $2)", &[&elem.amount, &table_id, &elem.name]) {
                    Ok(n) => res = format!("Update Item Successed! The item: {} table_id :{} amount: {} -> {}\n", elem.name, table_id, amount, elem.amount),
                    Err(e) => res = format!("Update Item Failed! The item: {} of table_id: {}, {}\n", elem.name, table_id, e)
                }               
            } else if item_status.eq("doing") {
                res = format!("Update Item Failed! The item: {} of table_id :{} is cooking\n", elem.name, table_id);               
            } else {
                res = format!("Update Item Failed! The item: {} of table_id: {} was done\n", elem.name, table_id);
            }
        },
        Err(err) => {
            res = format!("Update Item Successsed! New item: {} added to the table_id: {}\n", elem.name, table_id);
            // spawn another thread to process new item of the order of the table here
        }
    }
    // non-existent -> add the item; spawn another thread to process
    res
}