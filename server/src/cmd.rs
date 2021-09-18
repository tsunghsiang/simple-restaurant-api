use crate::db::DB;
use crate::order_type::{DeleteOrder, ItemPair, ItemStatus, PlaceOrder, TableStatus, UpdateOrder};
use crate::settings::Settings;
use chrono::{DateTime, Utc};
use futures::executor::block_on;
use postgres::{Client, Error, NoTls};
use rand::Rng;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct Dbio {
    name: String,
}

impl Dbio {
    pub fn new() -> Dbio {
        let mut db_url: String = "".to_string();
        let config: Settings = Settings::new();
        db_url.push_str(&config.database.get_prefix());
        db_url.push_str(":");
        db_url.push_str(&config.database.get_password());
        db_url.push_str("@");
        db_url.push_str(&config.database.get_ip());
        db_url.push_str(":");
        db_url.push_str(&config.database.get_port());
        db_url.push_str("/");
        db_url.push_str(&config.database.get_db_name());
        Dbio { name: db_url }
    }

    pub fn get_db_path(&self) -> &str {
        &self.name
    }
}

impl DB for Dbio {
    fn init(&self) -> Result<(), Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls)?;

        match client.query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'tablestatus')",
            &[],
        ) {
            Ok(row) => {
                let exists: bool = row.get(0);
                if exists == false {
                    client.batch_execute(
                        "CREATE TYPE TABLESTATUS AS ENUM (
                            'Open',
                            'Close'
                        )",
                    )?;
                }
            }
            _ => {}
        }

        match client.query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'itemstatus')",
            &[],
        ) {
            Ok(row) => {
                let exists: bool = row.get(0);
                if exists == false {
                    client.batch_execute(
                        "CREATE TYPE ITEMSTATUS AS ENUM (
                            'New',
                            'Process',
                            'Done',
                            'Deleted'
                        )",
                    )?;
                }
            }
            _ => {}
        }

        client.batch_execute(
            "
            CREATE TABLE IF NOT EXISTS TABLET (
                opened_at TIMESTAMPTZ,
                closed_at TIMESTAMPTZ,
                table_id VARCHAR,
                status TABLESTATUS
            );

            CREATE TABLE IF NOT EXISTS ITEMS (
                created_at TIMESTAMPTZ,
                updated_at TIMESTAMPTZ,
                table_id VARCHAR,
                item VARCHAR,
                amount INTEGER,
                status ITEMSTATUS
            );

            CREATE TABLE IF NOT EXISTS ITEM_HISTORY (
                created_at TIMESTAMPTZ,
                updated_at TIMESTAMPTZ,
                table_id VARCHAR,
                item VARCHAR,
                amount INTEGER,
                status ITEMSTATUS
            );
            ",
        )?;

        Ok(())
    }
    /*
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
                            client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES ($1, $2, $3, $4, $5)",
                                            &[&ts, &table_id, &item, &amount, &status])?;
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
                        client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES ($1, $2, $3, $4, $5)",
                                        &[&ts, &table_id, &item, &amount, &status])?;
                        // spawn a task handling item preparation
                        thread::spawn(move || block_on(cook_order_item(ts, &table_id.to_string(), &item.to_string(), cook_time)));
                    }
                }
            }

            Ok(res.into())
        }
    */

    /*
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
    */

    /*
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
    */

    fn query_by_tableid(&self, table_id: String) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        let mut res = "".to_owned();
        let ts: DateTime<Utc> = Utc::now();
        let mut empty: bool = true;
        res.push_str("{ queried_at: ");
        res.push_str(&ts.to_string());
        res.push_str(", table_id: ");
        res.push_str(&table_id);
        res.push_str(", items: [ ");
        for row in client.query("SELECT item, amount, status
                                 FROM items
                                 WHERE table_id = $1 AND created_at = (SELECT MAX(opened_at)
                                                                       FROM tablet
                                                                       WHERE table_id = $2) FOR UPDATE", &[&table_id, &table_id]).unwrap() {
            empty = false;
            let item: String = row.get(0);
            let amount: i32 = row.get(1);
            let status: ItemStatus = row.get(2);

            res.push_str("{ item: ");
            res.push_str(&item);
            res.push_str(", amount: ");
            res.push_str(&amount.to_string());
            res.push_str(", status: ");
            res.push_str(&status.to_string());
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
        let ts: DateTime<Utc> = Utc::now();
        let mut empty: bool = true;
        res.push_str("{ queried_at: ");
        res.push_str(&ts.to_string());
        for row in client.query("SELECT table_id, item, amount, status
                                 FROM items
                                 WHERE table_id = $1 AND item = $2 AND created_at = (SELECT MAX(opened_at)
                                                                                     FROM tablet
                                                                                     WHERE table_id = $1) FOR UPDATE", &[&table_id, &item]).unwrap() {
            empty = false;
            let table_id: String = row.get(0);
            let item: String = row.get(1);
            let amount: i32 = row.get(2);
            let status: ItemStatus = row.get(3);

            res.push_str(", table_id: ");
            res.push_str(&table_id);
            res.push_str(", item: ");
            res.push_str(&item);
            res.push_str(", amount: ");
            res.push_str(&amount.to_string());
            res.push_str(", item_status: ");
            res.push_str(&status.to_string());
        }

        if empty {
            res = format!("No Order of table id: {}", table_id);
        } else {
            res.push_str(" }");
        }

        Ok(res.into())
    }
    /*
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
    */
}

fn timestamp() -> i64 {
    let timespec = time::get_time();
    let mills: i64 = (timespec.sec as i64 * 1000) + (timespec.nsec as i64 / 1000 / 1000);
    mills
}
/*
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
*/

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_dbio_new_given_config_provided_when_init_then_inst_generated() {
        let dbio: Dbio = Dbio::new();
        assert!(dbio.get_db_path().len() > 0);
    }

    #[test]
    fn test_dbio_init_given_db_schema_setup_when_init_then_all_table_exist() {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                match client.query_one(
                    "SELECT EXISTS ( SELECT * FROM information_schema.tables WHERE table_name = 'tablet' )",
                    &[],
                ) {
                    Ok(row) => {
                        let exists: bool = row.get("exists");
                        assert!(exists);
                    }
                    Err(e) => panic!("[TEST::DBIO_INIT] Should not panic: {}", e),
                };
                match client.query_one(
                    "SELECT EXISTS ( SELECT * FROM information_schema.tables WHERE table_name = 'items' )",
                    &[],
                ) {
                    Ok(row) => {
                        let exists: bool = row.get("exists");
                        assert!(exists);
                    }
                    Err(e) => panic!("[TEST::DBIO_INIT] Should not panic: {}", e),
                };
                match client.query_one(
                    "SELECT EXISTS ( SELECT * FROM information_schema.tables WHERE table_name = 'item_history' )",
                    &[],
                ) {
                    Ok(row) => {
                        let exists: bool = row.get("exists");
                        assert!(exists);
                    }
                    Err(e) => panic!("[TEST::DBIO_INIT] Should not panic: {}", e),
                };
            }
            Err(e) => panic!("[TEST::DBIO_INIT] Should not panic: {}", e),
        };
    }
    /*
            #[test]
            fn test_dbio_check_table_status_given_no_rows_in_tablet_when_checked_then_true_returned() {
                let dbio:Dbio = Dbio::new();
                let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                match dbio.init() {
                    Ok(()) => {
                        // Clean both tablet/items tables
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                        client.execute("DELETE FROM items", &[]).unwrap();
                        match dbio.check_table_status() {
                            Ok(val) => assert_eq!(true, val),
                            Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Error: {}", e)
                        }
                    },
                    Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Should not panic: {}", e)
                };
            }

            #[test]
            fn test_dbio_check_table_status_given_all_table_status_of_tablet_rows_is_done_when_checked_then_true_returned() {
                let dbio:Dbio = Dbio::new();
                let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                match dbio.init() {
                    Ok(()) => {
                        // Clean both tablet/items tables
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                        client.execute("DELETE FROM items", &[]).unwrap();
                        client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'done')", &[]).unwrap();
                        match dbio.check_table_status() {
                            Ok(val) => assert_eq!(true, val),
                            Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Error: {}", e)
                        }
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                    },
                    Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Should not panic: {}", e)
                };
            }

            #[test]
            fn test_dbio_check_table_status_given_certain_table_status_of_tablet_rows_is_todo_when_checked_then_false_returned() {
                let dbio:Dbio = Dbio::new();
                let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                match dbio.init() {
                    Ok(()) => {
                        // Clean both tablet/items tables
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                        client.execute("DELETE FROM items", &[]).unwrap();
                        match client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'todo')", &[]) {
                            Ok(_) => {
                                match dbio.check_table_status() {
                                    Ok(val) => assert_eq!(false, val),
                                    Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Error: {}", e)
                                }
                            },
                            Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Error: {}", e)
                        }
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                    },
                    Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Should not panic: {}", e)
                };
            }

            #[test]
            fn test_dbio_check_table_status_given_certain_table_status_of_tablet_rows_is_doing_when_checked_then_false_returned() {
                let dbio:Dbio = Dbio::new();
                let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                match dbio.init() {
                    Ok(()) => {
                        // Clean both tablet/items tables
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                        client.execute("DELETE FROM items", &[]).unwrap();
                        client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'doing')", &[]).unwrap();
                        match dbio.check_table_status() {
                            Ok(val) => assert_eq!(false, val),
                            Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Error: {}", e)
                        }
                        client.execute("DELETE FROM tablet", &[]).unwrap();
                    },
                    Err(e) => panic!("[TEST::DBIO_CHECK_TABLE_STATUS] Should not panic: {}", e)
                };
            }
    */
    #[test]
    fn test_dbio_query_by_tableid_and_item_given_no_row_exists_when_select_then_result_contains_no_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                match dbio.query_by_tableid_and_item("1".to_string(), "A".to_string()) {
                    Ok(res) => assert!(res.contains("No")),
                    Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID_AND_ITEM] Error: {}", e),
                }
            }
            Err(e) => panic!(
                "[TEST::DBIO_QUERY_BY_TABLEID_AND_ITEM] Should not panic: {}",
                e
            ),
        };
    }

    #[test]
    fn test_dbio_query_by_tableid_and_item_given_one_row_exists_when_select_then_result_contains_item_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean tablet/items/item_history tables
                let opened_at = Utc::now();
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client
                    .execute(
                        "INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)",
                        &[&opened_at.to_string()],
                    )
                    .unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match dbio.query_by_tableid_and_item("1".to_string(), "A".to_string()) {
                    Ok(res) => assert!(res.contains("item")),
                    Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID_AND_ITEM] Error: {}", e),
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            }
            Err(e) => panic!(
                "[TEST::DBIO_QUERY_BY_TABLEID_AND_ITEM] Should not panic: {}",
                e
            ),
        };
    }
    #[test]
    fn test_dbio_query_by_tableid_given_no_tableid_exists_when_select_then_result_contains_no_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                match dbio.query_by_tableid("1".to_string()) {
                    Ok(res) => assert!(res.contains("No")),
                    Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID] Error: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_query_by_tableid_given_tableid_exists_when_select_then_result_contains_tableid_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at = Utc::now();
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client
                    .execute(
                        "INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)",
                        &[&opened_at.to_string()],
                    )
                    .unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match dbio.query_by_tableid("1".to_string()) {
                    Ok(res) => assert!(res.contains("table_id")),
                    Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID] Error: {}", e),
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            }
            Err(e) => panic!("[TEST::DBIO_QUERY_BY_TABLEID] Should not panic: {}", e),
        };
    }
    /*
        #[test]
        fn test_dbio_delete_given_no_row_when_delete_then_result_contains_failed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    let order: DeleteOrder = DeleteOrder {
                        timestamp: 1234567890123,
                        table_id: "1".to_string(),
                        item: "A".to_string()
                    };
                    match dbio.delete(order) {
                        Ok(res) => assert!(res.contains("Failed")),
                        Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e)
                    }
                },
                Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_delete_given_row_exists_and_state_doing_when_delete_then_result_contains_failed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'doing')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'doing')", &[]).unwrap();
                    let order: DeleteOrder = DeleteOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        item: "A".to_string()
                    };
                    match dbio.delete(order) {
                        Ok(res) => assert!(res.contains("Failed")),
                        Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_delete_given_row_exists_and_state_done_when_delete_then_result_contains_failed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'done')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'done')", &[]).unwrap();
                    let order: DeleteOrder = DeleteOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        item: "A".to_string()
                    };
                    match dbio.delete(order) {
                        Ok(res) => assert!(res.contains("Failed")),
                        Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_delete_given_row_exists_and_state_todo_when_delete_then_result_contains_successed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'todo')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'todo')", &[]).unwrap();
                    let order: DeleteOrder = DeleteOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        item: "A".to_string()
                    };
                    match dbio.delete(order) {
                        Ok(res) => assert!(res.contains("Successed")),
                        Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_update_given_no_row_exists_when_update_then_result_contains_non_existent_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    let order: UpdateOrder = UpdateOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "A".to_string(), amount: 1}]
                    };
                    match dbio.update(order) {
                        Ok(res) => assert!(res.contains("Non-existent")),
                        Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                    }
                },
                Err(e) => panic!("[TEST::DBIO_UPDATE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_update_given_table_status_done_when_update_then_result_contains_failed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'done')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'done')", &[]).unwrap();
                    let order: UpdateOrder = UpdateOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                    };
                    match dbio.update(order) {
                        Ok(res) => assert!(res.contains("Failed")),
                        Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_UPDATE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_update_given_table_status_todo_when_update_then_result_contains_successed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'todo')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'todo')", &[]).unwrap();
                    let order: UpdateOrder = UpdateOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                    };
                    match dbio.update(order) {
                        Ok(res) => assert!(res.contains("Successed")),
                        Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_UPDATE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_update_given_table_status_doing_when_update_then_result_contains_successed_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'doing')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'doing')", &[]).unwrap();
                    let order: UpdateOrder = UpdateOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                    };
                    match dbio.update(order) {
                        Ok(res) => assert!(res.contains("Successed")),
                        Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_UPDATE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_place_given_no_previous_row_when_place_then_result_contains_new_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    let order: PlaceOrder = PlaceOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                    };
                    match dbio.place(order) {
                        Ok(res) => assert!(res.contains("New")),
                        Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_place_given_previous_row_exists_and_table_status_todo_when_place_result_contains_duplicated_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'todo')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'todo')", &[]).unwrap();
                    let order: PlaceOrder = PlaceOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "B".to_string(), amount: 8}]
                    };
                    match dbio.place(order) {
                        Ok(res) => assert!(res.contains("Duplicated")),
                        Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_place_given_previous_row_exists_and_table_status_doing_when_place_result_contains_duplicated_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'doing')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'doing')", &[]).unwrap();
                    let order: PlaceOrder = PlaceOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "B".to_string(), amount: 8}]
                    };
                    match dbio.place(order) {
                        Ok(res) => assert!(res.contains("Duplicated")),
                        Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
            };
        }

        #[test]
        fn test_dbio_place_given_previous_row_exists_and_table_status_done_when_place_result_contains_new_string_literal() {
            let dbio:Dbio = Dbio::new();
            let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
            match dbio.init() {
                Ok(()) => {
                    // Clean both tablet/items tables
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                    client.execute("INSERT INTO tablet(timestamp, table_id, table_status) VALUES(1234567890123, '1', 'done')", &[]).unwrap();
                    client.execute("INSERT INTO items(timestamp, table_id, item, amount, item_status) VALUES(1234567890123, '1', 'A', 2, 'done')", &[]).unwrap();
                    let order: PlaceOrder = PlaceOrder {
                        timestamp: 1234567890124,
                        table_id: "1".to_string(),
                        items: vec![ItemPair{name: "B".to_string(), amount: 8}]
                    };
                    match dbio.place(order) {
                        Ok(res) => assert!(res.contains("New")),
                        Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                    }
                    client.execute("DELETE FROM tablet", &[]).unwrap();
                    client.execute("DELETE FROM items", &[]).unwrap();
                },
                Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
            };
        }
    */
}
