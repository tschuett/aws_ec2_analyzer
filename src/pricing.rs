use crate::pricing::regions::region2region;
use anyhow::{anyhow, Result};
use aws_sdk_ec2::types::SdkError;
use aws_sdk_pricing::{
    client,
    error::GetAttributeValuesError,
    model::{Filter, FilterType},
};
use serde_json::Value;
use tokio_stream::StreamExt;

mod regions {
    // FIXME may change
    // https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html#concepts-available-regions
    const REGIONS: &[(&str, &str)] = &[
        ("af-south-1", "Africa (Cape Town)"),
        ("ap-east-1", "Asia Pacific (Hong Kong)"),
        ("ap-south-1", "Asia Pacific (Mumbai)"),
        ("ap-northeast-3", "Asia Pacific (Osaka)"),
        ("ap-northeast-2", "Asia Pacific (Seoul)"),
        ("ap-southeast-1", "Asia Pacific (Singapore)"),
        ("ap-southeast-2", "Asia Pacific (Sydney)"),
        ("ap-northeast-1", "Asia Pacific (Tokyo)"),
        ("ca-central-1", "Canada (Central)"),
        ("eu-central-1", "EU (Frankfurt)"),
        ("eu-west-1", "EU (Ireland)"),
        ("eu-west-2", "EU (London)"),
        ("eu-south-1", "EU (Milan)"),
        ("eu-west-3", "EU (Paris)"),
        ("eu-north-1", "EU (Stockholm)"),
        ("me-south-1", "Middle East (Bahrain)"),
        ("sa-east-1", "South America (Sao Paulo)"),
        ("us-east-2", "US East (Ohio)"),
        ("us-east-1", "US East (N. Virginia)"),
        ("us-west-1", "US West (N. California)"),
        ("us-west-2", "US West (Oregon)"),
    ];

    // FIXME from from 1.56
    pub(super) fn region2region(region: &str) -> String {
        if let Some((_, r2)) = REGIONS.iter().find(|(r1, _)| *r1 == region) {
            return r2.to_string();
        }
        panic!("unknown region {}", region);
    }
}

#[derive(Debug)]
/// A wrapper around an AWS SDK Pricing client
pub struct Pricing(aws_sdk_pricing::client::Client);

impl Pricing {
    /// create new Pricing object taking an AWS Pricing client
    pub fn new(config: aws_types::SdkConfig) -> Self {
        let client = client::Client::new(&config);

        Pricing(client)
    }

    /// get all regions in AWS Pricing convention
    pub async fn get_regions(&self) -> std::result::Result<Vec<String>, SdkError<GetAttributeValuesError>> {
        let values = self
            .0
            .get_attribute_values()
            .service_code("AmazonEC2")
            .attribute_name("location")
            .into_paginator()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;

        let mut regions: Vec<String> = Vec::new();

        for region in values {
            for value in region.attribute_values().unwrap_or_default() {
                regions.push(value.value().unwrap_or_default().to_string())
            }
        }

        Ok(regions)
    }

    /// the OnDemand price for instance give a region
    pub async fn get_ondemand_price2(&self, instance: &str, region: &str) -> Result<f64> {
        let filters = self.get_ondemand_filters(instance, region);
        return self.get_price(instance, "OnDemand", &filters).await;
    }

    /// the Reservation price for instance give a region
    pub async fn get_reservation_price(&self, instance: &str, region: &str) -> Result<f64> {
        let filters = self.get_reservation_filters(instance, region);
        return self.get_price(instance, "Reserved", &filters).await;
    }

    /// the OnDemand price for instance give a region
    pub async fn get_ondemand_price(&self, instance: &str, region: &str) -> Result<f64> {
        let values = self
            .0
            .get_products()
            .service_code("AmazonEC2")
            .set_filters(Some(self.get_ondemand_filters(instance, region)))
            .into_paginator()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;

        let mut products = Vec::new();

        for page in values {
            for product in page.price_list().unwrap_or_default() {
                products.push(product.clone());
            }
        }

        if products.len() != 1 {
            return Err(anyhow!("not found: {instance}"));
        }

        let v: Value = serde_json::from_str(&products[0])?;

        let kv = v["terms"]["OnDemand"]
            .as_object()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        if kv.len() != 1 {
            return Err(anyhow!("not found: Ondemand"));
        }

        let kvs = kv[0].1["priceDimensions"]
            .as_object()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        if kvs.len() != 1 {
            return Err(anyhow!("not found: price"));
        }

        let mut value = kvs[0].1["pricePerUnit"]["USD"].to_string();
        value.pop();

        return Ok(value.strip_prefix('\"').unwrap().parse::<f64>()?);
    }

    fn get_common_filters(&self, instance: &str, region: &str, capacity_status: &str, tenancy: &str) -> Vec<Filter> {
        let location = region2region(region);
        vec![
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("instanceType")
                .value(instance)
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("location")
                .value(location)
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("operatingSystem")
                .value("Linux")
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("preInstalledSw")
                .value("NA")
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("operation")
                .value("RunInstances")
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("capacitystatus")
                .value(capacity_status)
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("tenancy")
                .value(tenancy)
                .build(),
        ]
    }
    fn get_ondemand_filters(&self, instance: &str, region: &str) -> Vec<Filter> {
        //let location = self.region2region(region);
        //vec![
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("instanceType".to_string()))
        //        .set_value(Some(instance.to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("location".to_string()))
        //        .set_value(Some(location))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("operatingSystem".to_string()))
        //        .set_value(Some("Linux".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("preInstalledSw".to_string()))
        //        .set_value(Some("NA".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("operation".to_string()))
        //        .set_value(Some("RunInstances".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("capacitystatus".to_string()))
        //        .set_value(Some("UnusedCapacityReservation".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("tenancy".to_string()))
        //        .set_value(Some("Shared".to_string()))
        //        .build(),
        //]
        self.get_common_filters(instance, region, "UnusedCapacityReservation", "Shared")
    }

    fn get_reservation_filters(&self, instance: &str, region: &str) -> Vec<Filter> {
        let mut filters = self.get_common_filters(instance, region, "Used", "Dedicated");
        let mut remainder = vec![
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("LeaseContractLength")
                .value("3yr")
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("OfferingClass")
                .value("standard")
                .build(),
            Filter::builder()
                .set_type(Some(FilterType::TermMatch))
                .field("PurchaseOption")
                .value("All Upfront")
                .build(),
        ];
        filters.append(&mut remainder);
        filters

        //vec![
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("instanceType".to_string()))
        //        .set_value(Some(instance.to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("location".to_string()))
        //        .set_value(Some(location))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("operatingSystem".to_string()))
        //        .set_value(Some("Linux".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("preInstalledSw".to_string()))
        //        .set_value(Some("NA".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("operation".to_string()))
        //        .set_value(Some("RunInstances".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("capacitystatus".to_string()))
        //        .set_value(Some("Used".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("tenancy".to_string()))
        //        .set_value(Some("Dedicated".to_string()))
        //        .build(),
        //    //self.get_common_filters(instance, region, "Used", "Dedicated")
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("LeaseContractLength".to_string()))
        //        .set_value(Some("3yr".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("OfferingClass".to_string()))
        //        .set_value(Some("standard".to_string()))
        //        .build(),
        //    Filter::builder()
        //        .set_type(Some(FilterType::TermMatch))
        //        .set_field(Some("PurchaseOption".to_string()))
        //        .set_value(Some("All Upfront".to_string()))
        //        .build(),
        ////]
    }

    /// the price for instance give a region
    async fn get_price(&self, instance: &str, price_kind: &str, filters: &[Filter]) -> Result<f64> {
        let values = self
            .0
            .get_products()
            .service_code("AmazonEC2")
            .set_filters(Some(filters.to_vec()))
            .into_paginator()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;

        let mut products: Vec<String> = Vec::new();

        for product in values {
            for item in product.price_list().unwrap_or_default() {
                products.push(item.clone());
            }
        }

        if products.len() != 1 {
            return Err(anyhow!("not found: {instance}"));
        }

        let v: Value = serde_json::from_str(&products[0])?;

        let kv = v["terms"][price_kind]
            .as_object()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        let kvs = kv[0].1["priceDimensions"]
            .as_object()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        if kvs.len() != 2 {
            println!("v = {}", v);
            println!("kv = {:?}", kv);
            println!("kvs = {:?}", kvs);
            println!("kvs.len() = {}", kvs.len());
            return Err(anyhow!("not found: price"));
        }

        let mut value = kvs[0].1["pricePerUnit"]["USD"].to_string();
        value.pop();
        return Ok(value.strip_prefix('\"').unwrap().parse::<f64>()?);
    }
}

fn _print_filters(filters: &[Filter]) {
    for filter in filters {
        println!(
            "{} = {}",
            filter.field.as_ref().unwrap(),
            filter.value.as_ref().unwrap()
        );
    }
}
