use anyhow::Result;
use aws_sdk_ec2::model::Filter;

#[allow(unused)]
async fn print_pcluster(client: &aws_sdk_ec2::Client) -> Result<()> {
    let results = client.describe_images().owners("247102896272").send().await?;

    for image in results.images().unwrap_or_default() {
        let location = image.image_location().unwrap();
        if location.starts_with("amazon/aws-parallelcluster") {
            if let Some(description) = image.description() {
                println!("{}", description);
            }
        }
    }

    Ok(())
}

#[allow(unused)]
async fn print_pcluster2(client: &aws_sdk_ec2::Client) -> Result<()> {
    let prefix = format!("aws-parallelcluster-{}-*", "3.0.2");

    let filter = Filter::builder().name("name").values(prefix).build();

    let results = client
        .describe_images()
        .owners("247102896272")
        .filters(filter)
        .send()
        .await?;

    Ok(())
}

/// print snapshot
async fn print_snapshot(client: &aws_sdk_ec2::Client, snapshot_id: &str) -> Result<()> {
    let result = client
        .describe_snapshots()
        .snapshot_ids(snapshot_id)
        .owner_ids("amazon")
        .send()
        .await?;

    if let Some(token) = result.next_token() {
        println!("next_token: {:}", token);
    }

    for snapshot in result.snapshots().unwrap_or_default() {
        if let Some(description) = snapshot.description() {
            println!("{}", description);
        }
    }

    Ok(())
}

/// print images
#[allow(unused)]
pub async fn print_images(client: &aws_sdk_ec2::Client) -> Result<()> {
    let filters = vec![Filter::builder().name("name").values("amzn*").build()];

    let result = client
        .describe_images()
        .owners("amazon")
        .set_filters(Some(filters))
        .send()
        .await?;

    let mut images = result.images.unwrap_or_default();

    images.sort_unstable_by(|a, b| {
        a.creation_date()
            .unwrap()
            .partial_cmp(b.creation_date().unwrap())
            .unwrap()
    });

    for image in images {
        if let Some(description) = image.description() {
            if description.starts_with("Amazon Linux 2") {
                println!("{}: {}", description, image.image_id().unwrap());
            }
        }
        if let Some(mapping) = image.block_device_mappings() {
            for map in mapping {
                if let Some(ebs) = map.ebs() {
                    if let Some(id) = ebs.snapshot_id() {
                        print_snapshot(client, id).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

// aws ec2 describe-images --image-ids ami-0419ee5fcafb90b57
// pcluster list-official-images

// aws ec2 describe-images --owners amazon --filters "Name=name,Values=amzn*" --query 'sort_by(Images, &CreationDate)[].Name'

// aws ec2 describe-images --image-ids ami-0bd99ef9eccfee250
