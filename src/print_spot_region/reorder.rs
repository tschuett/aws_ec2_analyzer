use crate::instance::Instance;
use crate::print_spot_region::spot_region::SpotRegion;
use aws_sdk_ec2::model::InstanceType;
use std::collections::HashMap;

pub(crate) fn reorder(
    regions: &[SpotRegion],
    instances: &[InstanceType],
) -> (Vec<SpotRegion>, Vec<String>, Vec<f64>) {
    let prices: HashMap<String, Option<f64>> = get_average_instance_prices(regions, instances);

    let mut reordered_instances = prices.iter().collect::<Vec<_>>();

    reordered_instances.sort_unstable_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    let mut reordered_instances = reordered_instances
        .iter()
        .map(|i| i.0.clone())
        .collect::<Vec<String>>();

    // big in front

    reordered_instances.reverse();

    let price_changes = find_price_changes(&reordered_instances, &prices);

    let reordered_regions = reorder_regions(regions, instances);

    (reordered_regions, reordered_instances, price_changes)
}

fn extract_instance_data(
    instance: InstanceType,
    regions: &[SpotRegion],
) -> Vec<(usize, Option<Instance>)> {
    let mut result: Vec<(usize, Option<Instance>)> = Vec::with_capacity(regions.len());

    // counter is row
    for (counter, region) in regions.iter().enumerate() {
        if let Some(item) = region.find_instance(instance.as_str()) {
            let item2 = item.clone();
            result.push((counter, Some(item2)));
        } else {
            result.push((counter, None));
        }
    }

    result
}

fn find_completes_instance(
    regions: &[SpotRegion],
    instance_types: &[InstanceType],
) -> InstanceType {
    let mut instances: HashMap<InstanceType, u32> = HashMap::new();

    for instance in instance_types {
        let mut count: u32 = 0;

        for region in regions {
            if region.contains(instance.as_str()) {
                count += 1;
            }
        }

        if count > 0 {
            instances.insert(instance.clone(), count);
        }
    }

    let mut counts = instances.drain().collect::<Vec<(InstanceType, u32)>>();

    counts.sort_unstable_by_key(|p| p.1);

    counts.pop().unwrap().0
}

fn reorder_regions(regions: &[SpotRegion], instances: &[InstanceType]) -> Vec<SpotRegion> {
    let most_complete = find_completes_instance(regions, instances);
    let instance_data = extract_instance_data(most_complete, regions);

    let mut data: Vec<(usize, f64)> = Vec::new();

    for id in &instance_data {
        match &id.1 {
            Some(inst) => {
                data.push((id.0, inst.get_ondemand_price()));
            }

            None => {
                data.push((id.0, 0.0));
            }
        }
    }

    data.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let rows = data.iter().map(|a| a.0).collect::<Vec<usize>>();

    reorder_data_by(&rows, regions)
}

fn get_average_instance_prices(
    regions: &[SpotRegion],
    instances: &[InstanceType],
) -> HashMap<String, Option<f64>> {
    let mut averages: HashMap<String, Option<f64>> = HashMap::new();

    for instance in instances {
        let mut sum: f64 = 0.0;
        let mut count: u32 = 0;

        for region in regions {
            if let Some(price) = region.find_instance(instance.as_str()) {
                let ondemand_price = price.get_ondemand_price();
                sum += ondemand_price;
                count += 1;
            }
        }

        if count > 0 {
            averages.insert(instance.as_str().to_string(), Some(sum / count as f64));
        } else {
            averages.insert(instance.as_str().to_string(), None);
        }
    }

    averages
}

fn find_price_changes(
    instances: &[String],
    average_prices: &HashMap<String, Option<f64>>,
) -> Vec<f64> {
    let mut changes = Vec::with_capacity(instances.len());

    for index in 0..instances.len() - 1 {
        let i0 = instances[index].clone();
        let i1 = instances[index + 1].clone();

        if let (Some(p0), Some(p1)) = (average_prices.get(&i0), average_prices.get(&i1)) {
            if let Some(pt1) = p1 {
                // has spot pricej
                if *pt1 == 0.0 {
                    println!("{i0} {i1}");
                    changes.push(0.0);
                }
                //} else {
            }
            if let Some(pt1) = p1 {
                if let Some(pt0) = p0 {
                    changes.push(pt0 / pt1);
                }
            }

            //if *p1 == 0.0 {
            //    println!("{} {}", i0, i1);
            //    changes.push(0.0);
            //} else {
            //    changes.push(p0 / p1);
            //}
        }
    }

    changes
}

fn reorder_data_by(rows: &[usize], regions: &[SpotRegion]) -> Vec<SpotRegion> {
    let mut result: Vec<SpotRegion> = Vec::with_capacity(rows.len());

    for row in rows {
        result.push(regions.get(*row).unwrap().clone());
    }

    result
}
