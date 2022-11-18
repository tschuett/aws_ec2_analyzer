use anyhow::Result;
use aws_sdk_ec2::model::SpotPlacementScore;
use tokio_stream::StreamExt;

/// prints spot placement scores
pub async fn print_spot_placement(instances: &[String], capacity: i32) -> Result<()> {
    let shared_config = aws_config::load_from_env().await;

    let ec2_client = aws_sdk_ec2::Client::new(&shared_config);

    let result: Result<Vec<_>, _> = ec2_client
        .get_spot_placement_scores()
        .set_instance_types(Some(instances.to_vec()))
        .set_target_capacity(Some(capacity))
        .set_single_availability_zone(Some(true))
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    let scores: Vec<SpotPlacementScore> = result?;

    for score in scores {
        if let Some(region) = score.region() {
            println!(
                "{}; {}; {}",
                region,
                score.availability_zone_id().unwrap_or_default(),
                score.score().unwrap_or_default()
            )
        }
    }

    println!("done");

    Ok(())
}
