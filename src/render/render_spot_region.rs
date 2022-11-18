use crate::get_region_config;
use crate::get_zones;
use crate::{pricing::Pricing, render::render_spot_zone::render_spot_zone};
use anyhow::Result;
use serde::Serialize;
use tera::Context;

#[derive(Serialize)]
struct DataSet {
    data: Vec<Zone>,
}

#[derive(Serialize)]
struct Zone {
    title: String,
    instances: Vec<Instance>,
}

#[derive(Serialize)]
struct Instance {
    instance: String,
    file: String,
    ondemand: f64,
}

/// render a gnuplot file with spot prices for one region
pub async fn render_spot_region(
    pricing: &Pricing,
    spot_instances: &[aws_sdk_ec2::model::InstanceType],
    region: &str,
    prefix: &str,
) -> Result<()> {
    let mut data = Vec::new();

    let config = get_region_config(region).await;
    let client = aws_sdk_ec2::Client::new(&config);

    let zones: Vec<String> = get_zones(&client, region).await?;

    for zone in zones {
        let mut instances = Vec::new();

        for instance in spot_instances {
            if let Ok(file) = render_spot_zone(&client, &zone, instance.clone(), prefix).await {
                match pricing.get_ondemand_price(instance.as_str(), region).await {
                    Ok(ondemand) => instances.push(Instance {
                        instance: instance.as_str().to_string(),
                        file,
                        ondemand,
                    }),
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            };
        }

        if !instances.is_empty() {
            let zone_data = Zone {
                title: zone.clone(),
                instances,
            };

            data.push(zone_data);
        }
    }

    let dataset = DataSet { data };

    let tera = {
        let mut tera = tera::Tera::default();
        tera.add_template_file("templates/render_spot_region.gnuplot", None)?;

        tera.autoescape_on(vec![".gnuplot"]);
        tera
    };

    let context = {
        let mut context = Context::new();

        context.insert("context", &dataset);
        context
    };

    let result = tera.render("templates/render_spot_region.gnuplot", &context)?;

    std::fs::write("render_spot_region.gnuplot", result).expect("Unable to write file");

    Ok(())
}
