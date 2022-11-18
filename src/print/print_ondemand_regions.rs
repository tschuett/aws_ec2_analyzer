use crate::{ec2::Ec2, pricing::Pricing};
use anyhow::Result;
use aws_sdk_ec2::model::InstanceType;

#[allow(unused)]
/// print the OnDemand prices for an instance in all regions
pub async fn print_ondemand_regions(pricing: &Pricing, ec2: &Ec2, instance: InstanceType) -> Result<()> {
    let regions = ec2.get_regions().await?;

    let mut prices = Vec::new();

    for region in regions {
        match pricing.get_ondemand_price(instance.as_str(), &region).await {
            Ok(price) => {
                prices.push((region, price));
            }
            Err(e) => {
                println!("error: {}; region: {}", e, region);
            }
        }
    }

    prices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (region, price) in prices {
        println!("{:30} : {}", region, price);
    }

    Ok(())
}
