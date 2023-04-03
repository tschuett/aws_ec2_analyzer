use crate::availability_zone::AvailabilityZone;
use crate::get_spot_price_history;
use crate::instance::Instance;
use crate::pricing::Pricing;
use crate::print_spot_region::spot_region::SpotRegion;
use anyhow::Result;
use aws_sdk_ec2::primitives::DateTime;
use aws_sdk_ec2::types::InstanceType;

pub(super) struct DataCollector {
    region: String,
    zones: Vec<String>,
    instances: Vec<InstanceType>,
}

impl DataCollector {
    pub(super) fn new(region: &str, zones: &[String], instances: &[InstanceType]) -> Self {
        DataCollector {
            region: region.to_string(),
            zones: zones.to_vec(),
            instances: instances.to_vec(),
        }
    }

    pub(super) async fn get_region(
        &self,
        client: &aws_sdk_ec2::Client,
        pricing: &Pricing,
    ) -> Result<SpotRegion> {
        let mut region_data: SpotRegion = SpotRegion::new(&self.region);

        for instance in &self.instances {
            if let Ok(spot_region) = self
                .get_zones(client, pricing, &self.zones, instance.clone())
                .await
            {
                region_data.add(instance.as_str(), spot_region);
            }
        }

        Ok(region_data)
    }

    async fn get_zones(
        &self,
        client: &aws_sdk_ec2::Client,
        pricing: &Pricing,
        zones: &[String],
        instance: InstanceType,
    ) -> Result<Instance> {
        let mut result_zones = Vec::new();

        for zone in zones {
            let spot_history = get_spot_price_history(client, zone, instance.clone()).await?;

            if spot_history.is_empty() {
                continue;
            }

            let min_entry = self.min(&spot_history);
            let max_entry = self.max(&spot_history);
            let avg_entry = self.average(&spot_history);

            let last = spot_history.last().unwrap().1;

            let zone = AvailabilityZone::new(min_entry, avg_entry, max_entry, last);

            result_zones.push(zone);
        }

        //if result_zones.is_empty() {
        //    return Err(anyhow!("no entries found"));
        //}

        let ondemand = pricing
            .get_ondemand_price(instance.as_str(), &self.region)
            .await?;

        let spot_region = Instance::new(&self.region, instance, &result_zones, ondemand);

        Ok(spot_region)
    }

    fn min(&self, data: &[(DateTime, f64)]) -> f64 {
        data.iter()
            .map(|e| e.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn max(&self, data: &[(DateTime, f64)]) -> f64 {
        data.iter()
            .map(|e| e.1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn average(&self, data: &[(DateTime, f64)]) -> f64 {
        let sum: f64 = data.iter().map(|e| e.1).sum();
        sum / data.len() as f64
    }
}
