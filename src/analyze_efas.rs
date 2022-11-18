use crate::get_region_config;
use anyhow::Result;
use aws_sdk_ec2::model::Filter;
use aws_sdk_ec2::model::InstanceType;
use itertools::Itertools;
use std::collections::HashMap;
use tokio_stream::StreamExt;

const LARGE_INSTANCES: &[InstanceType] = &[
    InstanceType::Hpc6a48xlarge,
    InstanceType::C6a48xlarge,
    InstanceType::M6a48xlarge,
    //InstanceType::R6a48xlarge,
    InstanceType::C6gn16xlarge,
    InstanceType::C7g16xlarge,
    InstanceType::X2idn32xlarge,
    InstanceType::X2iedn32xlarge,
    InstanceType::I4i32xlarge,
    InstanceType::Im4gn16xlarge,
    InstanceType::M6i32xlarge,
    InstanceType::R6i32xlarge,
    InstanceType::P4d24xlarge,
    InstanceType::G4dn16xlarge,
    InstanceType::Inf124xlarge,
    InstanceType::I3en24xlarge,
    InstanceType::C5n18xlarge,
    InstanceType::R5n24xlarge,
    InstanceType::R5dn24xlarge,
    InstanceType::M5n24xlarge,
    InstanceType::M5dn24xlarge,
    InstanceType::M5zn12xlarge,
    InstanceType::C6i32xlarge,
    InstanceType::M6id32xlarge,
    InstanceType::R6a48xlarge,
    InstanceType::R6id32xlarge,
    InstanceType::C6id32xlarge,
];

const SMALL_INSTANCES: &[InstanceType] = &[
    InstanceType::I3en12xlarge,
    InstanceType::C6a32xlarge,
    InstanceType::C5n9xlarge,
    InstanceType::G4dn12xlarge,
    InstanceType::G4dn8xlarge,
    InstanceType::M6a32xlarge,
    InstanceType::R6a32xlarge,
];

// m6id.32xlarge
// r6a.48xlarge
// r6id.32xlarge
// c6id.32xlarge

pub async fn analyze_efas(ec2: &aws_sdk_ec2::Client) -> Result<()> {
    let filters = vec![
        Filter::builder()
            .name("network-info.efa-supported")
            .set_values(Some(vec!["true".to_string()]))
            .build(),
        //            Filter::builder()
        //                .name("instance-storage-supported")
        //                .set_values(Some(vec!["false".to_string()]))
        //                .build(),
        Filter::builder()
            .name("bare-metal")
            .set_values(Some(vec!["false".to_string()]))
            .build(),
    ];

    let shared_config = get_region_config("us-east-2").await;
    let ec2 = aws_sdk_ec2::Client::new(&shared_config);

    let result = ec2
        .describe_instance_types()
        .set_filters(Some(filters))
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    // describe_instance(&ec2, InstanceType)

    let mut core_freq: HashMap<i32, Vec<InstanceType>> = HashMap::new();

    for instance_type_info in result {
        if let Some(vcpu_info) = instance_type_info.v_cpu_info() {
            if let Some(cores) = vcpu_info.default_cores() {
                if let Some(ins_type) = instance_type_info.instance_type() {
                    if let Some(count) = core_freq.get_mut(&cores) {
                        count.push(ins_type.clone());
                    } else {
                        core_freq.insert(cores, vec![ins_type.clone()]);
                    }
                    if let None = LARGE_INSTANCES.iter().position(|x| x == ins_type) {
                        if let None = SMALL_INSTANCES.iter().position(|x| x == ins_type) {
                            let str = ins_type.as_str();
                            println!("unknown large instance: {str}");
                        }
                    }
                }
                if cores < 30 {
                    if let Some(ins_type) = instance_type_info.instance_type() {
                        let str = ins_type.as_str();
                        //println!("{cores} {str}");
                    }
                }
            }
        }
    }

    for (core, instances) in core_freq.iter().sorted() {
        let instances_as_str = format(&instances);
        println!("{core}: {instances_as_str}");
    }

    Ok(())
}

//aws ec2 describe-instance-types \
//--region us-east-2 \
//--filters Name=network-info.efa-supported,Values=true \
//--query "InstanceTypes[*].[InstanceType]" \
//--output table

fn format(instances: &[InstanceType]) -> String {
    return instances
        .iter()
        .map(|i| i.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");
}
