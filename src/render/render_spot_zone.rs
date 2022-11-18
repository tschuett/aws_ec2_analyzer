use crate::get_spot_price_history;
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::Utc;
use std::{fs::File, io::Write};

pub(crate) async fn render_spot_zone(
    client: &aws_sdk_ec2::Client,
    zone: &str,
    instance: InstanceType,
    prefix: &str,
) -> Result<String> {
    let spot_history = get_spot_price_history(client, zone, instance.clone()).await?;

    let file_name = format!(
        "{}-{}-{}-{}.txt",
        prefix,
        zone,
        instance.as_str(),
        Utc::now().format("%d-%m-%Y")
    );
    let mut file = File::create(file_name.clone())?;

    if spot_history.is_empty() {
        return Err(anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "describe_zone failed",
        )));
    }

    for value in spot_history {
        let timestamp = value.0;
        let chrono_date_time: chrono::DateTime<Utc> = timestamp.to_chrono_utc();
        let pretty_timestamp = chrono_date_time.format("%Y-%m-%dT%H:%M:%S.000Z");
        let price = value.1;
        let line = format!("{} {}\n", pretty_timestamp, price).to_owned();
        file.write_all(line.as_bytes())?;
    }

    Ok(file_name)
}

//    println!("{} {:?} {}", instance, location, zone);
