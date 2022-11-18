use aws_sdk_ec2::model::InstanceType;
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Clone)]
pub(crate) struct Region {
    region: String,
    instance_prices: HashMap<InstanceType, f64>,
}

impl Region {
    pub(super) fn new(region: &str) -> Self {
        Region {
            region: region.to_string(),
            instance_prices: HashMap::default(),
        }
    }

    pub(super) fn add(&mut self, instance: InstanceType, cost: f64) {
        self.instance_prices.insert(instance, cost);
    }

    pub(super) fn contains(&self, instance: &InstanceType) -> bool {
        self.instance_prices.contains_key(instance)
    }

    pub(super) fn find_instance(&self, instance: &InstanceType) -> Option<&f64> {
        self.instance_prices.get(instance)
    }

    pub(super) fn get_region(&self) -> Cow<'static, str> {
        Cow::from(self.region.clone())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.instance_prices.is_empty()
    }
}

use crate::region::reorder::find_price_changes;
use crate::region::reorder::get_average_instance_prices;
use crate::region::reorder::reorder_regions;

pub(crate) fn reorder(regions: &[Region], instances: &[InstanceType]) -> (Vec<Region>, Vec<InstanceType>, Vec<f64>) {
    let prices = get_average_instance_prices(regions, instances);

    let mut reordered_instances = prices.iter().collect::<Vec<_>>();

    reordered_instances.sort_unstable_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    let mut reordered_instances = reordered_instances
        .iter()
        .map(|i| i.0.clone())
        .collect::<Vec<InstanceType>>();

    // big in front

    reordered_instances.reverse();

    let price_changes = find_price_changes(&reordered_instances, &prices);

    let reordered_regions = reorder_regions(regions, instances);

    (reordered_regions, reordered_instances, price_changes)
}

mod reorder {
    use super::Region;
    use aws_sdk_ec2::model::InstanceType;
    use std::collections::HashMap;

    fn extract_instance_data(instance: InstanceType, regions: &[Region]) -> Vec<(usize, f64)> {
        let mut result: Vec<(usize, f64)> = Vec::with_capacity(regions.len());

        // counter is row
        for (counter, region) in regions.iter().enumerate() {
            if let Some(item) = region.find_instance(&instance) {
                result.push((counter, *item));
            } else {
                result.push((counter, 0.0));
            }
        }

        result
    }

    fn find_completes_instance(regions: &[Region], instance_types: &[InstanceType]) -> InstanceType {
        let mut instances: HashMap<InstanceType, u32> = HashMap::new();

        for instance in instance_types {
            for region in regions {
                if region.contains(instance) {
                    match instances.get_mut(instance) {
                        Some(x) => {
                            *x += 1;
                        }
                        None => {
                            instances.insert(instance.clone(), 1);
                        }
                    }
                }
            }
        }

        let mut counts = instances.drain().collect::<Vec<(InstanceType, u32)>>();

        counts.sort_unstable_by_key(|p| p.1);

        counts.pop().unwrap().0
    }

    fn reorder_data_by(rows: &[usize], regions: &[Region]) -> Vec<Region> {
        let mut result: Vec<Region> = Vec::with_capacity(rows.len());

        for row in rows {
            result.push(regions.get(*row).unwrap().clone());
        }

        result
    }

    pub(super) fn reorder_regions(regions: &[Region], instances: &[InstanceType]) -> Vec<Region> {
        let most_complete = find_completes_instance(regions, instances);
        let mut instance_data = extract_instance_data(most_complete, regions);
        instance_data.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let rows = instance_data.iter().map(|a| a.0).collect::<Vec<usize>>();

        reorder_data_by(&rows, regions)
    }

    pub(super) fn get_average_instance_prices(
        regions: &[Region],
        instances: &[InstanceType],
    ) -> HashMap<InstanceType, f64> {
        let mut averages: HashMap<InstanceType, f64> = HashMap::new();

        for instance in instances {
            let mut sum: f64 = 0.0;
            let mut count: u32 = 0;

            for region in regions {
                if let Some(price) = region.find_instance(instance) {
                    sum += price;
                    count += 1;
                }
            }

            if count > 0 {
                averages.insert(instance.clone(), sum / count as f64);
            }
        }

        averages
    }

    pub(super) fn find_price_changes(
        instances: &[InstanceType],
        average_prices: &HashMap<InstanceType, f64>,
    ) -> Vec<f64> {
        let mut changes = Vec::with_capacity(instances.len());

        for index in 0..instances.len() - 1 {
            let i0 = instances[index].clone();
            let i1 = instances[index + 1].clone();

            if let (Some(p0), Some(p1)) = (average_prices.get(&i0), average_prices.get(&i1)) {
                if *p1 == 0.0 {
                    //println!("{} {}", i0, i1);
                    changes.push(0.0);
                } else {
                    changes.push(p0 / p1);
                }
            }
        }

        changes
    }
}
