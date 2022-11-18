use crate::{describe_instance, ec2::Ec2};
use anyhow::Result;

/// print instances but filtered by some condition
pub async fn print_filtered_instances(ec2: &Ec2, client: &aws_sdk_ec2::Client) -> Result<()> {
    let instances = ec2.get_instance_types().await?;

    for instance in instances.into_iter() {
        let result = describe_instance(client, instance.instance_type().unwrap().clone()).await?;
        if let Some(gpu) = result.gpu_info() {
            for gp in gpu.gpus().unwrap_or_default().iter() {
                if gp.manufacturer().unwrap() == "NVIDIA" {
                    println!("{}: {}", gp.name().unwrap(), gp.count().unwrap());
                }
            }
        }
    }

    Ok(())
}
