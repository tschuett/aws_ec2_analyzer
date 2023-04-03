use anyhow::Result;
use aws_sdk_ec2::client;
use aws_sdk_ec2::error::SdkError;
use aws_sdk_ec2::operation::describe_instance_types::DescribeInstanceTypesError;
use aws_sdk_ec2::operation::describe_regions::DescribeRegionsError;
use aws_sdk_ec2::types::Filter;
use aws_sdk_ec2::types::InstanceTypeInfo;
use tokio_stream::StreamExt;

#[derive(Debug)]
/// a wrapper around an AWS EC2 client
pub struct Ec2(aws_sdk_ec2::client::Client);

impl Ec2 {
    /// create a wrapper around an AWS EC2 client
    pub fn new(client: client::Client) -> Self {
        Ec2(client)
    }

    #[allow(unused)]
    /// get all current types of instances
    pub(crate) async fn get_instance_types(
        &self,
    ) -> Result<Vec<InstanceTypeInfo>, SdkError<DescribeInstanceTypesError>> {
        let instances = self
            .0
            .describe_instance_types()
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;

        Ok(instances)
    }

    /// get all instances with EFA support
    pub async fn get_instance_types_efa(
        &self,
    ) -> Result<Vec<InstanceTypeInfo>, SdkError<DescribeInstanceTypesError>> {
        let filters = vec![
            Filter::builder()
                .name("network-info.efa-supported")
                .set_values(Some(vec!["true".to_string()]))
                .build(),
            Filter::builder()
                .name("instance-storage-supported")
                .set_values(Some(vec!["false".to_string()]))
                .build(),
            Filter::builder()
                .name("bare-metal")
                .set_values(Some(vec!["false".to_string()]))
                .build(),
        ];

        let instances = self
            .0
            .describe_instance_types()
            .set_filters(Some(filters.clone()))
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await?;

        Ok(instances)
    }

    pub(crate) async fn get_regions(&self) -> Result<Vec<String>, SdkError<DescribeRegionsError>> {
        let result = self.0.describe_regions().all_regions(true).send().await?;

        // no paging !

        Ok(result
            .regions
            .unwrap_or_default()
            .into_iter()
            .filter(|region| {
                region.opt_in_status().as_ref().unwrap_or(&"") == &"opt-in-not-required"
            })
            .filter(|region| region.region_name().is_some())
            .map(|r| r.region_name().unwrap().to_string())
            .collect::<Vec<_>>())
    }

    //    #[allow(unused)]
    //    async fn get_max_spot_price_spread(
    //        &self,
    //        zone: &str,
    //        instances: &[InstanceType],
    //    ) -> Result<Vec<(String, f64)>> {
    //        let mut spread = Vec::<(String, f64)>::new();
    //
    //        for instance in instances {
    //            let mut prices = self.get_spot_price_history(instance.clone(), zone).await?;
    //            prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    //            if !prices.is_empty() {
    //                let min = prices.first().unwrap();
    //                let max = prices.last().unwrap();
    //                spread.push((instance.as_str().to_string(), max.1 / min.1));
    //            }
    //        }
    //
    //        spread.sort_by(|(_i1, s1), (_i2, s2)| s1.partial_cmp(s2).unwrap());
    //
    //        let (_, second) = spread.split_at(min(spread.len() - 10, spread.len()));
    //
    //        Ok(second.to_vec())
    //    }
}
