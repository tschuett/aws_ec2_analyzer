#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub,
    future_incompatible,
    rust_2021_compatibility,
    unused
)]
#![deny(
    clippy::as_conversions,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

//! various tools for analyzing prices of AWS EC2 instances

/// the AWS EC2 client
pub mod ec2;
/// the AWS pricing client
pub mod pricing;

/// print spot prices for the different regions
pub mod print_spot_region {

    /// print spot prices for the different regions
    pub mod print_spot_regions;

    mod data_collector;

    mod spot_region;

    mod reorder;

    mod printer;
}

/// print information about EC2 instances
pub mod print_instances;

mod availability_zone;
mod instance;

use aws_sdk_ec2::{
    error::DescribeInstanceTypesError,
    model::{InstanceType, InstanceTypeInfo},
    types::SdkError,
};

async fn describe_instance(
    client: &aws_sdk_ec2::Client,
    instance: InstanceType,
) -> Result<InstanceTypeInfo, SdkError<DescribeInstanceTypesError>> {
    let result = client
        .describe_instance_types()
        .instance_types(instance)
        .send()
        .await?;

    return Ok(result
        .instance_types()
        .unwrap_or_default()
        .first()
        .unwrap()
        .clone());
}

// FIXME: join_all + tokio::spawn

/// get a shared_config configured for a given region
pub async fn get_region_config(region: &str) -> aws_types::SdkConfig {
    aws_config::from_env()
        .region(aws_types::region::Region::new(region.to_string()))
        .load()
        .await
}

use aws_sdk_ec2::error::DescribeSpotPriceHistoryError;
use aws_sdk_ec2::types::DateTime;
use tokio_stream::StreamExt;

async fn get_spot_price_history(
    client: &aws_sdk_ec2::Client,
    availibility_zone: &str,
    instance_type: InstanceType,
) -> Result<Vec<(DateTime, f64)>, SdkError<DescribeSpotPriceHistoryError>> {
    let prices = client
        .describe_spot_price_history()
        .instance_types(instance_type)
        .product_descriptions("Linux/UNIX")
        .availability_zone(availibility_zone)
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    let spot_prices = prices
        .iter()
        .filter(|price| price.spot_price().is_some() && price.timestamp().is_some())
        .map(|price| {
            let time = *price.timestamp().unwrap(); //.to_chrono_utc()
            let spot_price = price.spot_price().as_ref().unwrap().parse::<f64>().unwrap();
            (time, spot_price)
        })
        .collect::<Vec<_>>();

    Ok(spot_prices)
}

use aws_sdk_ec2::error::DescribeAvailabilityZonesError;
use aws_sdk_ec2::model::Filter;

async fn get_zones(
    client: &aws_sdk_ec2::Client,
    region: &str,
) -> Result<Vec<String>, SdkError<DescribeAvailabilityZonesError>> {
    let filter = Filter::builder().name("region-name").values(region).build();

    // no paging !

    let result = client
        .describe_availability_zones()
        .filters(filter)
        .send()
        .await?;

    Ok(result
        .availability_zones
        .unwrap_or_default()
        .into_iter()
        .filter(|ava| ava.zone_name().is_some())
        .map(|ava| ava.zone_name().unwrap().to_string())
        .collect::<Vec<_>>())
}

fn get_padding(pad: char, len: usize) -> String {
    let mut s = String::default();

    s.reserve(len - s.len());

    while s.len() < len {
        s.push(pad);
    }

    s
}

fn get_string_with_len(s: &str, len: usize) -> String {
    get_string_with_len_and_padding(s, len, ' ')
}

fn get_f64_with_len(float: f64, len: usize) -> String {
    let s = format!("{float:.5}");

    get_string_with_len_and_padding(&s, len, ' ')
}

fn get_option_f64_with_len(float: Option<f64>, len: usize) -> String {
    if let Some(f) = float {
        let s = format!("{f:.5}");

        return get_string_with_len_and_padding(&s, len, ' ');
    }
    get_string_with_len_and_padding("", len, ' ')
}

use num_traits::PrimInt;
use std::fmt::Display;

fn get_integer_with_len<T: Display + PrimInt>(int: T, len: usize) -> String {
    let s = format!("{int:.5}");

    get_string_with_len_and_padding(&s, len, ' ')
}

fn get_string_with_len_and_padding(str: &str, len: usize, pad: char) -> String {
    let mut s = str.to_string();

    if s.len() > len {
        let two = s.split_at(len);

        return two.0.to_string();
    }

    s.reserve(len - s.len());

    while s.len() < len {
        s.push(pad);
    }

    s
}

fn get_string_network_and_len(s: &str, dot: usize, len: usize) -> String {
    match s.find('G') {
        Some(offset) => {
            if offset > dot {
                println!("{s} {dot} {len} {offset}");
            }
            let pre_padding = get_padding(' ', dot - offset);
            let s = format!("{pre_padding}{s}");
            get_string_with_len(&s, len)
        }
        None => get_string_with_len(s, len),
    }
}

//fn get_bool_with_len(b: bool, len: usize) -> String {
//    let s = format!("{0:.5}", b);
//
//    get_string_with_len_and_padding(&s, len, ' ')
//}

fn get_string_with_dot_and_len(s: &str, dot: usize, len: usize) -> String {
    match s.find('.') {
        Some(offset) => {
            let pre_padding = get_padding(' ', dot - offset);
            let s = format!("{pre_padding}{s}");
            get_string_with_len(&s, len)
        }
        None => get_string_with_len(s, len),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_bool_with_len() -> std::io::Result<()> {
        let s = get_bool_with_len(true, 5);
        assert_eq!(s.len(), 5);

        Ok(())
    }

    #[test]
    fn test_stackoverflow() -> std::io::Result<()> {
        let s = get_string_with_dot_and_len("im4gn.16xlarge", 7, 15);
        //assert_eq!(s.len(), 5);

        Ok(())
    }
}

//use aws_sdk_ec2::error::DescribeInstanceTypeOfferingsError;
//use aws_sdk_ec2::model::LocationType;

//pub async fn describe_instance_type_offerings(
//) -> Result<(), SdkError<DescribeInstanceTypeOfferingsError>> {
//    let shared_config = get_region_config("eu-north-1").await;
//    let client = aws_sdk_ec2::Client::new(&shared_config);
//
//    let filter = Filter::builder()
//        .name("instance-type")
//        .values("hpc6a.48xlarge")
//        .build();
//    let values = client
//        .describe_instance_type_offerings()
//        .location_type(LocationType::AvailabilityZone)
//        .filters(filter)
//        .into_paginator()
//        .items()
//        .send()
//        .collect::<Result<Vec<_>, _>>()
//        .await?;
//
//    for value in values {
//        println!("{}", value.location().unwrap());
//    }
//
//    Ok(())
//}

// aws ec2 describe-instance-type-offerings \
//--location-type availability-zone \
//--region eu-central-1 \
//--filters Name=instance-type,Values=p4d.24xlarge \
//--query "InstanceTypeOfferings[*].Location" \
//--output text
