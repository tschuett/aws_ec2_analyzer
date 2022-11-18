use crate::get_region_config;
use crate::pricing::Pricing;
use anyhow::Result;
use aws_sdk_ec2::{model::InstanceType, Region};
use chrono::DateTime;
use chrono::Utc;
use rand::prelude::*;
use rusqlite::Connection;

const PATH: &str = "aws_rc2.sql";

/// store ondemand prices in sqlite
pub async fn write_to_sql() -> Result<()> {
    let db = Connection::open(PATH)?;

    db.execute(
        "
    CREATE TABLE ondemand (
    id           INTEGER PRIMARY KEY,
    instancetype TEXT NOT NULL,
    date         TEXT NOT NULL,
    region       TEXT NOT NULL
)",
        (),
    )?;

    Ok(())
}

struct Item {
    id: i64,
    instance_type: InstanceType,
    date: DateTime<Utc>,
    region: Region,
    price: f64,
}

impl Item {
    fn get_instance_type(&self) -> String {
        self.instance_type.as_str().to_string()
    }

    fn get_region(&self) -> String {
        self.region.as_ref().to_string()
    }

    fn get_date(&self) -> String {
        let formatted = format!("{}", self.date.format("%Y-%m-%d:%H-%M-%S"));
        formatted
    }

    fn get_id(&self) -> String {
        self.id.to_string()
    }

    fn get_price(&self) -> String {
        self.price.to_string()
    }
}

struct Sqlite {
    conn: Connection,
}

impl Sqlite {
    fn new() -> Result<Self> {
        Ok(Sqlite {
            conn: Connection::open(PATH)?,
        })
    }

    fn create_table(&self) {
        let result = self.conn.execute(
            "
    CREATE TABLE ondemand (
    id            INTEGER PRIMARY KEY,
    instance_type TEXT NOT NULL,
    date          TEXT NOT NULL,
    region        TEXT NOT NULL
    price         NUM NOT NULL,
)",
            (),
        );
    }

    fn insert(&self, item: Item) {
        let result = self.conn.execute(
            "INSERT INTO ondemand (id, instance_type, date, region, price) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                item.get_id(),
                item.get_instance_type(),
                item.get_date(),
                item.get_region(),
                item.get_price()
            ),
        );
    }
}

/// add instances * regions into the sqlite database
pub async fn fill_db(instance_types: &[InstanceType], regions: &[Region]) -> Result<()> {
    let sql = Sqlite::new()?;
    sql.create_table();

    let pricing_config = get_region_config("us-east-1").await;
    let pricing = Pricing::new(pricing_config);

    let date = Utc::now();
    let mut rng = rand::thread_rng();

    for instance in instance_types {
        for region in regions {
            let price = pricing
                .get_ondemand_price(instance.as_str(), region.as_ref())
                .await?;

            let id: i64 = rng.gen();
            sql.insert(Item {
                id: id,
                instance_type: instance.clone(),
                date,
                region: region.clone(),
                price: price,
            });
        }
    }

    Ok(())
}
