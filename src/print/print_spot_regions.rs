use crate::ec2::Ec2;
use anyhow::{anyhow, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::{
    client, error::DescribeSpotPriceHistoryError, model::InstanceType, DateTime, SdkError,
};
use std::collections::HashMap;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
struct Region {
    region: String,
    instance_prices: HashMap<String, f64>,
}

impl Region {
    fn new(region: &str) -> Self {
        Region {
            region: region.to_string(),
            instance_prices: HashMap::default(),
        }
    }

    fn add(&mut self, instance: &str, cost: f64) {
        self.instance_prices.insert(instance.to_string(), cost);
    }

    fn contains(&self, instance: &str) -> bool {
        self.instance_prices.contains_key(instance)
    }

    fn find_instance(&self, instance: &str) -> Option<&f64> {
        self.instance_prices.get(instance)
    }

    fn get_region(&self) -> String {
        self.region.clone()
    }

    fn is_empty(&self) -> bool {
        self.instance_prices.is_empty()
    }

    fn add_averages(&self, prices: &mut HashMap<String, (f64, u32)>) {
        for (instance, price) in &self.instance_prices {
            match prices.get_mut(instance) {
                Some(x) => {
                    x.0 += price;
                    x.1 += 1;
                }
                None => {
                    prices.insert(instance.clone(), (*price, 1));
                }
            }
        }
    }
}

fn extract_instance_data(instance: &str, regions: &[Region]) -> Vec<(usize, f64)> {
    let mut result: Vec<(usize, f64)> = Vec::new();

    // counter is row
    for (counter, region) in regions.iter().enumerate() {
        if let Some(item) = region.find_instance(instance) {
            result.push((counter, *item));
        } else {
            result.push((counter, 0.0));
        }
    }

    result
}

#[allow(unused)]
fn average_price(data: &[Region]) -> Vec<String> {
    let mut prices: HashMap<String, (f64, u32)> = HashMap::new();

    let mut averages: Vec<(String, f64)> = Vec::new();

    for region in data {
        region.add_averages(&mut prices);
    }

    for (instance, (sum, count)) in prices {
        averages.push((instance.clone(), sum / count as f64));
    }

    averages.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    averages.reverse();

    // big in front

    averages
        .iter()
        .map(|f| f.0.clone())
        .collect::<Vec<String>>()
}

fn find_completes_instance(regions: &[Region], instance_types: &[&str]) -> String {
    let mut instances: HashMap<String, u32> = HashMap::new();

    for instance in instance_types {
        for region in regions {
            if region.contains(instance) {
                match instances.get_mut(&instance.to_string()) {
                    Some(x) => {
                        *x += 1;
                    }
                    None => {
                        instances.insert(instance.to_string(), 1);
                    }
                }
            }
        }
    }

    let mut counts = instances.drain().collect::<Vec<(String, u32)>>();

    counts.sort_unstable_by_key(|p| p.1);

    counts.pop().unwrap().0
}

fn reorder_data_by(rows: &[usize], data: &[Region]) -> Vec<Region> {
    let mut result: Vec<Region> = Vec::new();

    for row in rows {
        result.push(data.get(*row).unwrap().clone());
    }

    result
}

/// print the Spot prices for an instance in all regions
pub async fn print_spot_regions(ec2: &Ec2, instances: &[&str]) -> Result<()> {
    let mut data: Vec<Region> = Vec::new();

    let regions = ec2.get_regions().await?;

    data.try_reserve(regions.len())?;

    for region in regions {
        let zones = ec2.get_zones(&region).await?;
        let collector = DataCollector::new(&region, &zones, instances);
        let region_data = collector.get_region().await?;
        data.push(region_data);
    }

    let most_complete = find_completes_instance(&data, instances);
    let mut instance_data = extract_instance_data(&most_complete, &data);
    instance_data.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let rows = instance_data.iter().map(|a| a.0).collect::<Vec<usize>>();

    let reorderd_data = reorder_data_by(&rows, &data);

    let render = Printer::new(&reorderd_data, instances);

    render.render();

    //render(&reorderd_data, &average_price(&data));

    Ok(())
}

struct Printer {
    regions: Vec<Region>,
    instances: Vec<String>,
}

impl Printer {
    fn new(regions: &[Region], instances: &[&str]) -> Self {
        Self {
            regions: regions.to_vec(),
            instances: instances.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        }
    }

    fn render(&self) {
        let region_width = self.widest_region(&self.regions);
        let instance_width = self.widest_instance(&self.instances);

        print!("{} |", self.get_string_with_len("", region_width));
        for instance in &self.instances {
            print!(" {} |", self.get_string_with_len(&instance, instance_width));
        }
        println!();
        println!(
            "{}",
            self.get_string_with_len_and_padding(
                "-",
                region_width + 2 + (instance_width + 3) * self.instances.len(),
                '-',
            )
        );

        for region in &self.regions {
            if region.is_empty() {
                continue;
            }
            print!(
                "{} |",
                self.get_string_with_len(&region.get_region(), region_width)
            );
            for instance in &self.instances {
                if let Some(el) = region.find_instance(&instance) {
                    print!(
                        " {} |",
                        self.get_string_with_len(&el.to_string(), instance_width)
                    );
                } else {
                    print!(" {} |", self.get_string_with_len("", instance_width));
                }
            }
            println!();
        }
    }

    fn widest_region(&self, data: &[Region]) -> usize {
        data.iter().map(|el| el.get_region().len()).max().unwrap()
    }

    fn widest_instance(&self, instances: &[String]) -> usize {
        instances.iter().map(|el| el.len()).max().unwrap()
    }

    fn get_string_with_len(&self, str: &str, len: usize) -> String {
        let mut data = str.to_string();

        data.reserve(len - data.len());

        while data.len() < len {
            data.push(' ');
        }

        data
    }

    fn get_string_with_len_and_padding(&self, str: &str, len: usize, pad: char) -> String {
        let mut data = str.to_string();

        data.reserve(len - data.len());

        while data.len() < len {
            data.push(pad);
        }

        data
    }
}

struct DataCollector {
    region: String,
    zones: Vec<String>,
    instances: Vec<String>,
}

impl DataCollector {
    fn new(region: &str, zones: &[String], instances: &[&str]) -> Self {
        DataCollector {
            region: region.to_string(),
            zones: zones.to_vec(),
            instances: instances.iter().map(|i| i.to_string()).collect::<Vec<_>>(),
        }
    }

    async fn get_region(&self) -> Result<Region> {
        let mut region_data: Region = Region::new(&self.region);
        let client = self.get_region_provider(&self.region).await; // must be used!

        for instance in &self.instances {
            if let Ok(min) = self.get_zones(&client, &self.zones, &instance).await {
                region_data.add(&instance, min);
            }
        }

        Ok(region_data)
    }

    async fn get_region_provider(&self, region: &str) -> client::Client {
        let region_provider =
            RegionProviderChain::first_try(aws_types::region::Region::new(region.to_string()))
                .or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;

        aws_sdk_ec2::Client::new(&shared_config)
    }

    async fn get_spot_price_history(
        &self,
        client: &client::Client,
        availibility_zone: &str,
        instance: &str,
    ) -> Result<Vec<(DateTime, f64)>, SdkError<DescribeSpotPriceHistoryError>> {
        let paginator = client
            .describe_spot_price_history()
            .instance_types(InstanceType::from(instance))
            .product_descriptions("Linux/UNIX")
            .availability_zone(availibility_zone)
            .into_paginator()
            .items()
            .send();

        let prices = paginator.collect::<Result<Vec<_>, _>>().await?;

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

    async fn get_zones(
        &self,
        client: &client::Client,
        zones: &[String],
        instance: &str,
    ) -> Result<f64> {
        let mut min = f64::MAX;

        for zone in zones {
            let spot_history = self.get_spot_price_history(client, zone, instance).await?;
            if spot_history.is_empty() {
                continue;
            }
            let entry = spot_history.last().unwrap();
            let last_cost = entry.1;
            if last_cost < min {
                min = last_cost;
            }
        }

        if min == f64::MAX {
            Err(anyhow!("failed to find a value"))
        } else {
            Ok(min)
        }
    }
}
