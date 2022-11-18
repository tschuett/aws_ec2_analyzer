use crate::ec2::Ec2;
use crate::get_region_config;
use crate::get_zones;
use crate::pricing::Pricing;
use crate::print_spot_region::data_collector::DataCollector;
use crate::print_spot_region::printer::Printer;
use crate::print_spot_region::reorder::reorder;
use crate::print_spot_region::spot_region::SpotRegion;
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;

/// print the Spot prices for an instance in all regions
pub async fn print_spot_regions(
    ec2: &Ec2,
    pricing: &Pricing,
    instances: &[InstanceType],
    favorite_regions: &[&str],
) -> Result<()> {
    let mut regions: Vec<SpotRegion> = Vec::new();

    let region_names: Vec<String> = ec2.get_regions().await?;

    regions.try_reserve(region_names.len())?;

    let mut local_instances: Vec<InstanceType> = Vec::new();
    for instance in instances {
        local_instances.push(instance.clone());
    }

    for region in region_names {
        let config = get_region_config(&region).await;
        let client = aws_sdk_ec2::Client::new(&config);

        let zones = get_zones(&client, &region).await?;
        let collector = DataCollector::new(&region, &zones, instances);
        let region_data = collector.get_region(&client, pricing).await?;
        regions.push(region_data);
    }

    let regions_and_instances = reorder(&regions, instances);

    let printer = Printer::new(
        &regions_and_instances.0,
        &regions_and_instances.1,
        &regions_and_instances.2,
        favorite_regions,
    );

    printer.print();

    Ok(())
}
