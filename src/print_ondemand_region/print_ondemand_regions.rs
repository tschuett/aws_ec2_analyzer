use crate::{
    ec2::Ec2,
    pricing::Pricing,
    printer::Printer,
    region::{reorder, Region},
};
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;

/// print the Ondemand prices for instances in all regions
pub async fn print_ondemand_regions(pricing: &Pricing, ec2: &Ec2, instances: &[InstanceType]) -> Result<()> {
    let mut data: Vec<Region> = Vec::new();

    let regions = ec2.get_regions().await?;

    for region_name in &regions {
        let mut region_data = Region::new(region_name);
        for instance in instances {
            if let Ok(price) = pricing.get_ondemand_price(instance.as_str(), region_name).await {
                region_data.add(instance.clone(), price);
            }
        }
        data.push(region_data);
    }

    let regions_and_instances = reorder(&data, instances);

    let printer = Printer::new(
        &regions_and_instances.0,
        &regions_and_instances.1,
        &regions_and_instances.2,
    );

    printer.print();

    Ok(())
}
