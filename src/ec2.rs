use anyhow::Result;
use aws_sdk_ec2::client;
use aws_sdk_ec2::types::Filter;
use aws_sdk_ec2::types::InstanceTypeInfo;

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
    pub(crate) async fn get_instance_types(&self) -> Result<Vec<InstanceTypeInfo>> {
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
    pub async fn get_instance_types_efa(&self) -> Result<Vec<InstanceTypeInfo>> {
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

    pub(crate) async fn get_regions(&self) -> Result<Vec<String>> {
        //let result = self.0.describe_regions().all_regions(true).send().await?;

        // no paging !

        let _regions = vec![
            "Africa (Cape Town)",
            "Asia Pacific (Hong Kong)",
            "Asia Pacific (Melbourne)",
            "Europe (Zurich)",
            "Europe (Milan)",
            "Europe (Spain)",
            "Israel (Tel Aviv)",
            "Asia Pacific (Tokyo)",
            "Asia Pacific (Seoul)",
            "Asia Pacific (Osaka)",
            "Asia Pacific (Mumbai)",
            "Asia Pacific (Singapore)",
            "Asia Pacific (Sydney)",
            "Canada (Central",
            "Europa (Central)",
            "Europa (Frankfurt)",
            "Europa (Stockholm)",
            "Europa (Ireland)",
            "Europa (London)",
            "Europa (Paris)",
            "South America (São Paulo)",
            "US East (N. Virginia)",
            "US East (Ohio)",
            "US West (N. California)",
            "US West (Oregon)",
        ];

        let codes: Vec<String> = vec![
            "af-south-1".to_string(),
            "ap-east-1".to_string(),
            "eu-central-2".to_string(),
            "eu-south-1".to_string(),
            //"eu-south-2".to_string(),
            "il-central-1".to_string(),
            "ap-northeast-1".to_string(),
            //"ap-northeast-4".to_string(),
            "ap-northeast-3".to_string(),
            "ap-south-1".to_string(),
            "ap-southeast-1".to_string(),
            "ap-southeast-2".to_string(),
            "ca-central-1".to_string(),
            "eu-central-1".to_string(),
            "eu-north-1".to_string(),
            "eu-west-1".to_string(),
            "eu-west-2".to_string(),
            "eu-west-3".to_string(),
            "sa-east-1".to_string(),
            "us-east-1".to_string(),
            "us-east-2".to_string(),
            "us-west-1".to_string(),
            "us-west-2".to_string(),
        ];

        Ok(codes)

        //        Ok(result
        //            .regions
        //            .unwrap_or_default()
        //            .into_iter()
        //            .filter(|region| {
        //                region.opt_in_status().as_ref().unwrap_or(&"") == &"opt-in-not-required"
        //            })
        //            .filter(|region| region.region_name().is_some())
        //            .map(|r| r.region_name().unwrap().to_string())
        //            .collect::<Vec<_>>())
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
