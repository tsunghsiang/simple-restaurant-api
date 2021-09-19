use crate::db::DB;
use crate::order_type::{DeleteOrder, ItemPair, ItemStatus, PlaceOrder, TableStatus, UpdateOrder};
use crate::settings::Settings;
use chrono::{DateTime, Utc};
use futures::executor::block_on;
use postgres::{Client, Error, NoTls};
use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

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
  
    fn place(&self, order: PlaceOrder) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls)?;
        let mut res: String = "".to_string();

        match client.query_one("SELECT table_id, status
                                FROM tablet WHERE table_id = $1 AND opened_at = (SELECT MAX(opened_at)
                                                                                 FROM tablet
                                                                                 WHERE table_id = $1) FOR UPDATE", &[&order.table_id]) {
            Ok(row) => {
                let table_id: String = row.get(0);
                let status: TableStatus = row.get(1);

                match status {
                    TableStatus::Open => res = format!("Duplicated! There has been an order of table_id: {}, being served in the queue", table_id),
                    TableStatus::Close => {
                        res = format!("New Order! opened_at: {}, table_id: {}", order.created_at, order.table_id);
                        // insert new order into table 'tablet'
                        client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES ($1, $2, $3)", &[&order.created_at, &order.table_id, &TableStatus::Open])?;
                        // insert new items into table items/item_history
                        let mut rng = rand::thread_rng();
                        for elem in order.items {
                            let (ts, table_id, item, amount, cook_time) = (order.created_at, order.table_id.to_string(), elem.name, elem.amount, rng.gen_range(5..16));
                            client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'Process'::itemstatus)",
                                            &[&ts, &table_id, &item, &amount])?;
                            client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'New'::itemstatus)",
                                            &[&ts, &table_id, &item, &amount])?;
                            client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'Process'::itemstatus)",
                                            &[&ts, &table_id, &item, &amount])?;
                            // spawn a task handling item preparation
                            let elm = ItemPair{ name: item, amount: amount };
                            thread::spawn(move || cook_order_item(ts.to_string(), table_id, elm, cook_time));
                        }                       
                    }
                }
            },
            Err(_err) => {
                res = format!("New Order! opened_at_at: {}, table_id: {}", order.created_at, order.table_id);
                // insert new order into table 'tablet'
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES ($1, $2, $3)", &[&order.created_at, &order.table_id, &TableStatus::Open])?;
                // insert new items into table items/item_history
                let mut rng = rand::thread_rng();
                for elem in order.items {
                    let (ts, table_id, item, amount, cook_time) = (order.created_at, order.table_id.to_string(), elem.name, elem.amount, rng.gen_range(5..16));
                    client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'Process'::itemstatus)",
                                    &[&ts, &table_id, &item, &amount])?;
                    client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'New'::itemstatus)",
                                    &[&ts, &table_id, &item, &amount])?;
                    client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES ($1, $1, $2, $3, $4, 'Process'::itemstatus)",
                                    &[&ts, &table_id, &item, &amount])?;
                    // spawn a task handling item preparation
                    let elm = ItemPair{ name: item, amount: amount };
                    thread::spawn(move || cook_order_item(ts.to_string(), table_id, elm, cook_time));
                }
            }
        }

        Ok(res.into())
    }
  
    fn update(&self, order: UpdateOrder) -> Result<String, Error> {
        let mut client = Client::connect(self.get_db_path(), NoTls).unwrap();
        let mut res: String = "".to_string();

        match client.query_one("SELECT opened_at, table_id, status
                                FROM tablet
                                WHERE table_id = $1 AND opened_at = (SELECT MAX(opened_at)
                                                                     FROM tablet
                                                                     WHERE table_id = $1) FOR UPDATE", &[&order.table_id]) {
                Ok(row) => {
                    let opened_at: DateTime<Utc> = row.get("opened_at");
                    let table_id: String = row.get("table_id");
                    let status: TableStatus = row.get("status");
                    
                    match status {
                        TableStatus::Open => {
                            for elem in order.items {
                                let id = table_id.clone();
                                let updated_at = order.updated_at.to_string();
                                thread::spawn(move || update_order_item(opened_at.to_string(), updated_at, id, elem));
                            }
                            res = format!("Update Order Successed! table_id: {}", table_id);
                        },
                        TableStatus::Close => res = format!("Update Order Failed! table_id: {} was done. Please launch a new order", table_id)
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
        match client.query_one("SELECT created_at, table_id, item, amount, status
                                FROM items
                                WHERE table_id = $1 AND item = $2 and created_at = ( SELECT MAX(opened_at)
                                                                                     FROM tablet
                                                                                     WHERE table_id = $1 ) FOR UPDATE", &[&order.table_id, &order.item]){
                Ok(row) => {
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let table_id: String = row.get("table_id");
                    let item: String = row.get("item");
                    let amount: i32 = row.get("amount");
                    let status: ItemStatus = row.get("status");

                    match status {
                        ItemStatus::New => {
                            match client.execute("DELETE FROM items
                                                  WHERE table_id = $1 AND 
                                                        item = $2 AND 
                                                        status = 'New'::itemstatus AND 
                                                        created_at = (SELECT MAX(opened_at)
                                                                      FROM tablet
                                                                      WHERE table_id = $1)", &[&order.table_id, &order.item]) {
                                Ok(_n) => {
                                    client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                                    VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, 'Deleted'::itemstatus)", 
                                                    &[&created_at.to_string(), &order.deleted_at.to_string(), &table_id, &item, &amount]).unwrap();
                                    res = format!("Delete Order Successed! item: {} of table_id: {} deleted", order.item, order.table_id);
                                    update_table_status(client, table_id, created_at.to_string())?;
                                },
                                Err(_err) => {
                                    res = format!("Delete Order Failed! No item: {} of table_id: {}", order.item, order.table_id);
                                }
                            }
                        },
                        ItemStatus::Process => res = format!("Delete Order Failed! The item: {} of table_id: {} is cooking", item, table_id),
                        ItemStatus::Done => res = format!("Delete Order Failed! The item: {} of table_id: {} was done", item, table_id),
                        ItemStatus::Deleted => res = format!("Delete Order Failed! The item: {} of table_id: {} was deleted", item, table_id),
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

fn update_table_status(mut client: Client, table_id: String, ts: String) -> Result<(), Error> {
    let mut empty: bool = true;
    let mut open = false;

    for row in client.query(
        "SELECT status FROM items WHERE table_id = $1 AND created_at = (SELECT MAX(opened_at)
                                                                        FROM tablet
                                                                        WHERE table_id = $1) FOR UPDATE",
        &[&table_id],
    )? {
        empty = false;
        let status: ItemStatus = row.get(0);
        match status {
            ItemStatus::New => open = true,
            ItemStatus::Process => open = true,
            ItemStatus::Done => {}
            ItemStatus::Deleted => {}
        }
    }

    if empty {
        client.execute("UPDATE tablet 
                        SET closed_at = now(),
                            status = 'Close'::tablestatus
                        WHERE table_id = $1 AND opened_at = (SELECT MAX(opened_at)
                                                             FROM tablet
                                                             WHERE table_id = $1)",
            &[&table_id],
        )?;
        ()
    }

    if open {
        client.execute(
            "UPDATE tablet
             SET status = 'Open'::tablestatus
             WHERE table_id = $1 and opened_at = (SELECT MAX(opened_at)
                                                  FROM tablet
                                                  WHERE table_id = $1)",
            &[&table_id],
        )?;
    } else {
        client.execute(
            "UPDATE tablet
             SET closed_at = now(),
             status = 'Close'::tablestatus
             WHERE table_id = $1 and opened_at = (SELECT MAX(opened_at)
                                                  FROM tablet
                                                  WHERE table_id = $1)",
            &[&table_id],
        )?;
    }

    Ok(())
}

fn update_order_item(opened_at: String, updated_at: String, table_id: String, elem: ItemPair) {
    let command: Dbio = Dbio::new();
    let mut client = Client::connect(command.get_db_path(), NoTls).unwrap();
    let mut res: String = "".to_string();
    match client.query_one("SELECT created_at, table_id, item, amount, status
                            FROM items
                            WHERE table_id = $1 AND item = $2 AND created_at = (SELECT MAX(opened_at)
                                                                                FROM tablet
                                                                                WHERE table_id = $1) FOR UPDATE", &[&table_id, &elem.name]) {
        Ok(row) => {
            let status: ItemStatus = row.get("status");
            let amount: i32 = row.get("amount");

            match status {
                ItemStatus::New => {
                    client.execute("UPDATE items
                                    SET updated_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'),
                                    amount = $2
                                    WHERE table_id = $3 AND item = $4 AND created_at = (SELECT MAX(opened_at)
                                                                                        FROM tablet
                                                                                        WHERE table_id = $3))", &[&updated_at, &elem.amount, &table_id, &elem.name]).unwrap();
                    client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, ammount, status)
                                    VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&opened_at, &updated_at, &table_id, &elem.name, &elem.amount, &ItemStatus::New]).unwrap();
                    res = format!("Updat Item Successed! The item: {} of table_id: {} amount: {} -> {}", elem.name, table_id, amount, elem.amount);
                },
                ItemStatus::Process => res = format!("Update Item Failed! The item: {} of table_id :{} is cooking", elem.name, table_id),
                ItemStatus::Done => res = format!("Update Item Failed! The item: {} of table_id: {} was done", elem.name, table_id), 
                _ => {}
            }
        },
        Err(_err) => {
            let mut rng = rand::thread_rng();
            let cook_time: u64 = rng.gen_range(5..16);
            res = format!("Update Item Successed! New item: {} added to the table_id: {}", elem.name, table_id);
            // spawn a task handling item preparation
            std::thread::spawn(move || {
                let mut duration: Duration;
                // Start preparing food 
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, 'Process'::itemstatus)", &[&opened_at, &updated_at, &table_id, &elem.name, &elem.amount]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, 'New'::itemstatus)", &[&opened_at, &updated_at, &table_id, &elem.name, &elem.amount]).unwrap();               
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, 'Process'::itemstatus)", &[&opened_at, &updated_at, &table_id, &elem.name, &elem.amount]).unwrap();               
                let start = Instant::now(); 
                while start.elapsed().as_secs() < cook_time {}
                let done_at: String = Utc::now().to_string();
                let id: String = table_id;
                update_item_status(opened_at, done_at, id, elem, ItemStatus::Done);
            });
        }
    }

    // println!("{}", res);
}

fn update_item_status(created_at: String, updated_at: String, table_id: String, elem: ItemPair, to: ItemStatus) {
    let command: Dbio = Dbio::new();
    let mut client = Client::connect(command.get_db_path(), NoTls).unwrap();
    match client.execute("UPDATE items
                          SET updated_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'),
                              status = $2
                          WHERE table_id = $3 AND item = $4 AND created_at = (SELECT MAX(opened_at) FROM tablet WHERE table_id = $3)", &[&updated_at, &to, &table_id, &elem.name]) {
        Ok(_n) => {
            // println!("[UPDATE_ITEM_STATUS] {} rows modified", n);
            client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                            VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&created_at, &updated_at, &table_id, &elem.name, &elem.amount, &to]).unwrap();
        },
        Err(err) => println!("[UPDATE_ITEM_STATUS] Cook Error: {}", err)
    };
    match update_table_status(client, table_id, created_at) {
        Ok(()) => {},
        Err(_) => {}
    };
}

fn cook_order_item(ts: String, table_id: String, elem: ItemPair, cook_time: u64) {
    // println!("[COOK][START] table_id: {} item: {} cook_time: {} secs", table_id, elem.name, cook_time);
    let start = Instant::now();
    while start.elapsed().as_secs() < cook_time {
        // println!("wait...");
    }
    let done_at: String = Utc::now().to_string();
    update_item_status(ts, done_at, table_id, elem, ItemStatus::Done);
}

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
    fn test_dbio_update_table_status_given_no_items_left_when_executed_then_tablestatus_gets_close()
    {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                match update_table_status(client, "1".to_string(), opened_at.to_string()) {
                    Ok(()) => {
                        let mut cli: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                        match cli.query_one("SELECT opened_at, closed_at, table_id, status
                                             FROM tablet
                                             WHERE table_id = '1' AND opened_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')", &[&opened_at.to_string()]) {
                            Ok(row) => {
                                let closed_at: DateTime<Utc> = row.get(1);
                                let status: TableStatus = row.get(3);
                                assert!(closed_at.to_string().len() > 0, "Field 'clsoed_at' should not be null");
                                assert_eq!(TableStatus::Close, status, "tablestatus should be 'Close'");
                            },
                            Err(e) => {
                                panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] No row selected: {}", e)
                            }
                        }
                        cli.execute("DELETE FROM tablet", &[]).unwrap();
                    }
                    Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Err: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_update_table_status_given_all_itemstatus_done_when_executed_then_tablestatus_gets_close(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'A', 1, 'Done'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'B', 1, 'Done'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match update_table_status(client, "1".to_string(), opened_at.to_string()) {
                    Ok(()) => {
                        let mut cli: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                        match cli.query_one("SELECT opened_at, closed_at, table_id, status
                                             FROM tablet
                                             WHERE table_id = '1' AND opened_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')", &[&opened_at.to_string()]) {
                            Ok(row) => {
                                let closed_at: DateTime<Utc> = row.get(1);
                                let status: TableStatus = row.get(3);
                                assert!(closed_at.to_string().len() > 0, "Field 'clsoed_at' should not be null");
                                assert_eq!(TableStatus::Close, status, "tablestatus should be 'Close'");
                            },
                            Err(e) => {
                                panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] No row selected: {}", e)
                            }
                        }
                        cli.execute("DELETE FROM tablet", &[]).unwrap();
                        cli.execute("DELETE FROM items", &[]).unwrap();
                    }
                    Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Err: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_update_table_status_given_all_itemstatus_deleted_when_executed_then_tablestatus_gets_close(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'A', 1, 'Deleted'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'B', 1, 'Deleted'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match update_table_status(client, "1".to_string(), opened_at.to_string()) {
                    Ok(()) => {
                        let mut cli: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                        match cli.query_one("SELECT opened_at, closed_at, table_id, status
                                             FROM tablet
                                             WHERE table_id = '1' AND opened_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')", &[&opened_at.to_string()]) {
                            Ok(row) => {
                                let closed_at: DateTime<Utc> = row.get(1);
                                let status: TableStatus = row.get(3);
                                assert!(closed_at.to_string().len() > 0, "Field 'clsoed_at' should not be null");
                                assert_eq!(TableStatus::Close, status, "tablestatus should be 'Close'");
                            },
                            Err(e) => {
                                panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] No row selected: {}", e)
                            }
                        }
                        cli.execute("DELETE FROM tablet", &[]).unwrap();
                        cli.execute("DELETE FROM items", &[]).unwrap();
                    }
                    Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Err: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_update_table_status_given_certain_itemstatus_new_when_executed_then_tablestatus_gets_open(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'A', 1, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'B', 1, 'Done'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match update_table_status(client, "1".to_string(), opened_at.to_string()) {
                    Ok(()) => {
                        let mut cli: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                        match cli.query_one("SELECT opened_at, closed_at, table_id, status
                                             FROM tablet
                                             WHERE table_id = '1' AND opened_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')", &[&opened_at.to_string()]) {
                            Ok(row) => {
                                let status: TableStatus = row.get(3);
                                assert_eq!(TableStatus::Open, status, "tablestatus should be 'Open'");
                            },
                            Err(e) => {
                                panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] No row selected: {}", e)
                            }
                        }
                        cli.execute("DELETE FROM tablet", &[]).unwrap();
                        cli.execute("DELETE FROM items", &[]).unwrap();
                    }
                    Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Err: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_update_table_status_given_certain_itemstatus_process_when_executed_then_tablestatus_gets_open(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'A', 1, 'Process'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT into items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), to_timestamp($1, 'yyyy-mm-dd hh24:mi:ss'), '1', 'B', 1, 'Done'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                match update_table_status(client, "1".to_string(), opened_at.to_string()) {
                    Ok(()) => {
                        let mut cli: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
                        match cli.query_one("SELECT opened_at, closed_at, table_id, status
                                             FROM tablet
                                             WHERE table_id = '1' AND opened_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')", &[&opened_at.to_string()]) {
                            Ok(row) => {
                                let status: TableStatus = row.get(3);
                                assert_eq!(TableStatus::Open, status, "tablestatus should be 'Open'");
                            },
                            Err(e) => {
                                panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] No row selected: {}", e)
                            }
                        }
                        cli.execute("DELETE FROM tablet", &[]).unwrap();
                        cli.execute("DELETE FROM items", &[]).unwrap();
                    }
                    Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Err: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_UPDATE_TABLE_STATUS] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_update_item_status_given_an_item_created_when_updated_then_status_new() {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                
                let now: String = Utc::now().to_string();
                let created_at = now.clone();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 1}]
                };
                let table_id = order.table_id.clone();
                let updated_at: String = order.updated_at.to_string();
                let elem = ItemPair {
                    name: order.items[0].name.clone(),
                    amount: order.items[0].amount
                };
                let item = elem.name.clone();

                client.execute("INSERT INTO tablet(opened_at, table_id, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), $2, $3)", &[&now, &order.table_id, &TableStatus::Open]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                
                update_item_status(now, updated_at, order.table_id, elem, ItemStatus::New);

                match client.query_one("SELECT updated_at, status 
                                        FROM items
                                        WHERE created_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') AND table_id = $2 AND item = $3", &[&created_at, &table_id, &item]) {
                    Ok(row) => {
                        let updated_at: DateTime<Utc> = row.get(0);
                        let status: ItemStatus = row.get(1);
                        assert!(updated_at.to_string().len() > 0);
                        assert_eq!(ItemStatus::New, status);
                    },
                    Err(e) => panic!("[TEST::DBIO_UPDATE_ITEM_STATUS] Err: {}", e),
                }

                client.batch_execute(
                    "DELETE FROM tablet;
                     DELETE FROM items;
                     DELETE FROM item_history;").unwrap();
            }
            Err(e) => panic!(
                "[TEST::DBIO_UPDATE_ITEM_STATUS] Should not panic: {}",
                e
            ),
        };
    }
    
    #[test]
    fn test_dbio_update_item_status_given_an_item_process_when_updated_then_status_process() {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                
                let now: String = Utc::now().to_string();
                let created_at = now.clone();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 1}]
                };
                let table_id = order.table_id.clone();
                let updated_at: String = order.updated_at.to_string();
                let elem = ItemPair {
                    name: order.items[0].name.clone(),
                    amount: order.items[0].amount
                };
                let item = elem.name.clone();

                client.execute("INSERT INTO tablet(opened_at, table_id, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), $2, $3)", &[&now, &order.table_id, &TableStatus::Open]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                
                update_item_status(now, updated_at, order.table_id, elem, ItemStatus::Process);

                match client.query_one("SELECT updated_at, status 
                                        FROM items
                                        WHERE created_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') AND table_id = $2 AND item = $3", &[&created_at, &table_id, &item]) {
                    Ok(row) => {
                        let updated_at: DateTime<Utc> = row.get(0);
                        let status: ItemStatus = row.get(1);
                        assert!(updated_at.to_string().len() > 0);
                        assert_eq!(ItemStatus::Process, status);
                    },
                    Err(e) => panic!("[TEST::DBIO_UPDATE_ITEM_STATUS] Err: {}", e),
                }

                client.batch_execute(
                    "DELETE FROM tablet;
                     DELETE FROM items;
                     DELETE FROM item_history;").unwrap();
            }
            Err(e) => panic!(
                "[TEST::DBIO_UPDATE_ITEM_STATUS] Should not panic: {}",
                e
            ),
        };       
    }
    
    #[test]
    fn test_dbio_update_item_status_given_an_item_served_when_updated_then_status_done() {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items/item_history tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                
                let now: String = Utc::now().to_string();
                let created_at = now.clone();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 1}]
                };
                let table_id = order.table_id.clone();
                let updated_at: String = order.updated_at.to_string();
                let elem = ItemPair {
                    name: order.items[0].name.clone(),
                    amount: order.items[0].amount
                };
                let item = elem.name.clone();

                client.execute("INSERT INTO tablet(opened_at, table_id, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), $2, $3)", &[&now, &order.table_id, &TableStatus::Open]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status)
                                VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), $3, $4, $5, $6)", &[&now, &updated_at, &order.table_id, &order.items[0].name, &order.items[0].amount, &ItemStatus::New]).unwrap();
                
                update_item_status(now, updated_at, order.table_id, elem, ItemStatus::Done);

                match client.query_one("SELECT updated_at, status 
                                        FROM items
                                        WHERE created_at = to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') AND table_id = $2 AND item = $3", &[&created_at, &table_id, &item]) {
                    Ok(row) => {
                        let updated_at: DateTime<Utc> = row.get(0);
                        let status: ItemStatus = row.get(1);
                        assert!(updated_at.to_string().len() > 0);
                        assert_eq!(ItemStatus::Done, status);
                    },
                    Err(e) => panic!("[TEST::DBIO_UPDATE_ITEM_STATUS] Err: {}", e),
                }

                client.batch_execute(
                    "DELETE FROM tablet;
                     DELETE FROM items;
                     DELETE FROM item_history;").unwrap();
            }
            Err(e) => panic!(
                "[TEST::DBIO_UPDATE_ITEM_STATUS] Should not panic: {}",
                e
            ),
        };       
    }

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

    #[test]
    fn test_dbio_delete_given_no_items_when_delete_then_result_contains_failed_string_literal() {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                let order: DeleteOrder = DeleteOrder {
                    deleted_at: Utc::now(),
                    table_id: "1".to_string(),
                    item: "A".to_string(),
                };
                match dbio.delete(order) {
                    Ok(res) => assert!(res.contains("Failed")),
                    Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e),
                }
            }
            Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_delete_given_items_exist_and_status_process_when_delete_then_result_contains_failed_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                // Clean both tablet/items tables
                let opened_at: DateTime<Utc> = Utc::now();
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'Process'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                let order: DeleteOrder = DeleteOrder {
                    deleted_at: Utc::now(),
                    table_id: "1".to_string(),
                    item: "A".to_string(),
                };
                match dbio.delete(order) {
                    Ok(res) => assert!(res.contains("Failed")),
                    Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e),
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
            }
            Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_delete_given_items_exist_and_status_done_when_delete_then_result_contains_failed_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'Done'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                let order: DeleteOrder = DeleteOrder {
                    deleted_at: Utc::now(),
                    table_id: "1".to_string(),
                    item: "A".to_string(),
                };
                match dbio.delete(order) {
                    Ok(res) => assert!(res.contains("Failed")),
                    Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e),
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
            }
            Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e),
        };
    }

    #[test]
    fn test_dbio_delete_given_items_exist_and_status_new_when_delete_then_result_contains_successed_string_literal(
    ) {
        let dbio: Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: DateTime<Utc> = Utc::now();
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'Open'::tablestatus)", &[&opened_at.to_string()]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, 'New'::itemstatus)", &[&opened_at.to_string()]).unwrap();
                let order: DeleteOrder = DeleteOrder {
                    deleted_at: Utc::now(),
                    table_id: "1".to_string(),
                    item: "A".to_string(),
                };
                match dbio.delete(order) {
                    Ok(res) => assert!(res.contains("Successed")),
                    Err(e) => panic!("[TEST::DBIO_DELETE] Error: {}", e),
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            }
            Err(e) => panic!("[TEST::DBIO_DELETE] Should not panic: {}", e),
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
                client.execute("DELETE FROM item_history", &[]).unwrap();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
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
    fn test_dbio_update_given_table_status_close_when_update_then_result_contains_failed_string_literal() {
        let dbio:Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: String = Utc::now().to_string();
                let created_at: String = opened_at.clone();
                let update_at: String = created_at.clone();
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', $2)", &[&opened_at, &TableStatus::Close]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $3)", &[&created_at, &update_at, &ItemStatus::Done]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $3)", &[&created_at, &update_at, &ItemStatus::Done]).unwrap();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                };
                match dbio.update(order) {
                    Ok(res) => assert!(res.contains("Failed")),
                    Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            },
            Err(e) => panic!("[TEST::DBIO_UPDATE] Should not panic: {}", e)
        };
    }

    #[test]
    fn test_dbio_update_given_table_status_open_when_update_then_result_contains_successed_string_literal() {
        let dbio:Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                let opened_at: String = Utc::now().to_string();
                let created_at: String = opened_at.clone();
                let update_at: String = created_at.clone();
                // Clean both tablet/items tables
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', $2)", &[&opened_at, &TableStatus::Open]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $3)", &[&created_at, &update_at, &ItemStatus::New]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $3)", &[&created_at, &update_at, &ItemStatus::New]).unwrap();
                let order: UpdateOrder = UpdateOrder {
                    updated_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                };
                match dbio.update(order) {
                    Ok(res) => assert!(res.contains("Successed")),
                    Err(e) => panic!("[TEST::DBIO_UPDATE] Error: {}", e)
                }
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
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
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();

                let order: PlaceOrder = PlaceOrder {
                    created_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "A".to_string(), amount: 8}]
                };

                match dbio.place(order) {
                    Ok(res) => assert!(res.contains("New")),
                    Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                }
            },
            Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
        }          
    }

    #[test]
    fn test_dbio_place_given_previous_row_exists_and_table_status_open_when_place_result_contains_duplicated_string_literal() {
        let dbio:Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
                
                let opened_at: String = Utc::now().to_string();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', $2)", &[&opened_at, &TableStatus::Open]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $2)", &[&opened_at, &ItemStatus::New]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $2)", &[&opened_at, &ItemStatus::New]).unwrap();
                
                let order: PlaceOrder = PlaceOrder {
                    created_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "B".to_string(), amount: 8}]
                };
                    
                match dbio.place(order) {
                    Ok(res) => assert!(res.contains("Duplicated")),
                    Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                }
                
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            },
            Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
        };
    }

    #[test]
    fn test_dbio_place_given_previous_row_exists_and_table_status_close_when_place_result_contains_new_string_literal() {
        let dbio:Dbio = Dbio::new();
        let mut client: Client = Client::connect(dbio.get_db_path(), NoTls).unwrap();
        match dbio.init() {
            Ok(()) => {
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();

                let opened_at: String = Utc::now().to_string();
                client.execute("INSERT INTO tablet(opened_at, table_id, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', $2)", &[&opened_at, &TableStatus::Close]).unwrap();
                client.execute("INSERT INTO items(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $2)", &[&opened_at, &ItemStatus::Done]).unwrap();
                client.execute("INSERT INTO item_history(created_at, updated_at, table_id, item, amount, status) VALUES(to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'), '1', 'A', 2, $2)", &[&opened_at, &ItemStatus::Done]).unwrap();
            
                let order: PlaceOrder = PlaceOrder {
                    created_at: Utc::now(),
                    table_id: "1".to_string(),
                    items: vec![ItemPair{name: "B".to_string(), amount: 8}]
                };

                match dbio.place(order) {
                    Ok(res) => assert!(res.contains("New")),
                    Err(e) => panic!("[TEST::DBIO_PLACE] Error: {}", e)
                }
            
                client.execute("DELETE FROM tablet", &[]).unwrap();
                client.execute("DELETE FROM items", &[]).unwrap();
                client.execute("DELETE FROM item_history", &[]).unwrap();
            },
            Err(e) => panic!("[TEST::DBIO_PLACE] Should not panic: {}", e)
        };
    }
    
}
