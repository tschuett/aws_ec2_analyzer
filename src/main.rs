use anyhow::Result;
use aws_ec2_analyzer::{ec2::Ec2, get_region_config, pricing::Pricing};
//print_ondemand_region
//use aws_sdk_ec2::model::InstanceType;
//use aws_ec2_spot::print::print_spot_placement::print_spot_placement;
//use aws_ec2_spot::print_spot_region::print_spot_regions::print_spot_regions;
//use aws_ec2_analyzer::describe_instance_type_offerings;
use aws_sdk_ec2::model::InstanceType;

const REGIONS: &[&str] = &[
    "us-east-1",
    "us-east-2",
    "us-west-2",
    "eu-west-1",
    "eu-north-1",
];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let INSTANCE_TYPES: &[InstanceType] = &[
        //InstanceType::C524xlarge,
        InstanceType::C5n18xlarge,
        InstanceType::C6i32xlarge,
        InstanceType::C6a48xlarge,
        InstanceType::C6g16xlarge,
        InstanceType::C6gn16xlarge,
        InstanceType::C7g16xlarge,
        InstanceType::Hpc6a48xlarge,
        //InstanceType::M524xlarge,
        //InstanceType::M5n24xlarge,
        InstanceType::M6i32xlarge,
        InstanceType::M6a48xlarge,
        InstanceType::M6g16xlarge,
        // accelerator
        InstanceType::G548xlarge,
        //        InstanceType::C5Metal,
        //        InstanceType::C6iMetal,
        //        InstanceType::C6aMetal,
        //        InstanceType::M5znMetal,
        //        InstanceType::C5nMetal,
        //InstanceType::G54xlarge,
        //InstanceType::P4de24xlarge,
        //InstanceType::P4d24xlarge,
        //InstanceType::P3dn24xlarge,
        //InstanceType::G4ad16xlarge,
        //InstanceType::G4dnMetal,
        //InstanceType::G5gMetal,
        //InstanceType::C7g4xlarge,
        // memory
        //InstanceType::X2iedn32xlarge,
        //InstanceType::R6i32xlarge,
        //nstanceType::X2idn32xlarge,
        //InstanceType::R6g16xlarge,
        //InstanceType::R5b24xlarge,
        //InstanceType::X2gd16xlarge,
        //nstanceType::R5b24xlarge,
        //InstanceType::U12tb1112xlarge,
        //InstanceType::U24tb1Metal,
        //InstanceType::R6a48xlarge,
        // storage
        //InstanceType::Im4gn16xlarge,
        //InstanceType::I3en24xlarge,
        //InstanceType::I4i32xlarge,
        //InstanceType::Unknown("trn1.32xlarge".to_string()),
        //InstanceType::Unknown("inf2.48xlarge".to_string()),
        //InstanceType::Unknown("inf1.24xlarge".to_string()),
        //InstanceType::Inf124xlarge,
        InstanceType::Unknown("c6in.32xlarge".to_string()),
        InstanceType::Unknown("hpc6id.32xlarge".to_string()),
        InstanceType::Unknown("c7gn.16xlarge".to_string()),
        InstanceType::Unknown("r7iz.32xlarge".to_string()),
        InstanceType::Unknown("hpc7g.16xlarge".to_string()),
    ];

    let shared_config = aws_config::load_from_env().await;

    let ec2_client = aws_sdk_ec2::Client::new(&shared_config);

    let ec2 = Ec2::new(ec2_client);

    let pricing_config = get_region_config("us-east-1").await;
    let pricing = Pricing::new(pricing_config);

    aws_ec2_analyzer::print_spot_region::print_spot_regions::print_spot_regions(
        &ec2,
        &pricing,
        INSTANCE_TYPES,
        REGIONS,
    )
    .await?;

    aws_ec2_analyzer::print_instances::print_instances(INSTANCE_TYPES).await?;

    //print_spot_regions(&ec2, "c6gn.16xlarge").await?;
    //println!();
    //print_spot_regions(&ec2, "m6i.32xlarge").await?;
    //println!();
    //print_spot_regions(&ec2, "c6i.32xlarge").await?;
    ////println!();
    //    print_spot_regions(&ec2, "g5.48xlarge").await?;
    //    println!();
    //    print_spot_regions(&ec2, "g5.24xlarge").await?;
    //    println!();
    //    print_spot_regions(&ec2, "g5.16xlarge").await?;
    //    println!();
    //    print_spot_regions(&ec2, "g5.12xlarge").await?;
    //    println!();
    //    print_spot_regions(&ec2, "g5.4xlarge").await?;

    //
    //print_spot_regions(
    //    &ec2,
    //    &["g5.48xlarge", "g5.24xlarge", "g5.16xlarge", "g5.4xlarge"],
    //)
    //.await?;

    //render_spot_region(
    //    &ec2,
    //    &pricing,
    //    &[
    //        InstanceType::from("m6i.32xlarge"),
    //        InstanceType::from("m5.24xlarge"),
    //        InstanceType::from("m5a.24xlarge"),
    //        InstanceType::from("m6g.16xlarge"),
    //        InstanceType::from("c5n.18xlarge"),
    //        InstanceType::from("c6gn.16xlarge"),
    //    ],
    //    "us-east-2",
    //    "meds",
    //)
    //.await?;

    Ok(())
}

/*
When you specify a start and end time, this operation returns the
prices of the instance types within the time range that you specified
and the time when the price changed. The price is valid within the
time period that you specified; the response merely indicates the last
time that the price changed.
*/

/*
aws ec2 describe-instance-types \
--region us-east-2 \
--filters Name=network-info.efa-supported,Values=true \
--query "InstanceTypes[*].[InstanceType]" \
--output text
*/

/*
aws ec2 describe-instance-types \
--region us-east-2 \
--filters Name=network-info.efa-supported,Values=true \
--query "InstanceTypes[*].[InstanceType]" \
--output table
*/

/*
aws ec2 describe-instance-types \
--filters Name=network-info.encryption-in-transit-supported,Values=true \
--query "InstanceTypes[*].[InstanceType]" --output text
*/

// aws ec2 describe-images --image-ids ami-0419ee5fcafb90b57
// pcluster list-official-images

// aws ec2 describe-images --owners amazon --filters "Name=name,Values=amzn*" --query 'sort_by(Images, &CreationDate)[].Name'

// https://aws.amazon.com/ec2/spot/instance-advisor/
// https://docs.aws.amazon.com/whitepapers/latest/cost-optimization-leveraging-ec2-spot-instances/cost-optimization-leveraging-ec2-spot-instances.html
