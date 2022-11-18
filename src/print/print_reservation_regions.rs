use crate::{ec2::Ec2, pricing::Pricing};
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;

use crate::{
    printer::Printer,
    region::{reorder, Region},
};

#[allow(unused)]
/// print the Reservation prices for an instance in all regions
pub async fn print_reservation_regions(pricing: &Pricing, ec2: &Ec2, instances: &[InstanceType]) -> Result<()> {
    let mut data: Vec<Region> = Vec::new();

    let regions = ec2.get_regions().await?;

    for region_name in &regions {
        let mut region_data = Region::new(region_name);
        for instance in instances {
            if let Ok(price) = pricing.get_reservation_price(instance.as_str(), region_name).await {
                region_data.add(instance.clone(), price);
            }
        }
        data.push(region_data);
    }

    //    let reordered_data = reorder_regions(&data, instances);
    //    let render = Printer::new(&reordered_data, instances);

    let new_typed_regions = regions.iter().map(|r| Region::new(r)).collect::<Vec<_>>();

    let regions_and_instances = reorder(&new_typed_regions, instances);

    let printer = Printer::new(
        &regions_and_instances.0,
        &regions_and_instances.1,
        &regions_and_instances.2,
    );

    printer.print();

    Ok(())
}
