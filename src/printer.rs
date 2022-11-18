use crate::get_f64_with_len;
use crate::get_string_with_len;
use crate::get_string_with_len_and_padding;
use crate::region::Region;
use aws_sdk_ec2::model::InstanceType;

mod regions {
    // FIXME may change
    // https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html#concepts-available-regions
    const REGIONS: &[(&str, &str)] = &[
        ("af-south-1", "Cape Town"),
        ("ap-east-1", "Hong Kong"),
        ("ap-south-1", "Mumbai"),
        ("ap-northeast-3", "Osaka"),
        ("ap-northeast-2", "Seoul"),
        ("ap-southeast-1", "Singapore"),
        ("ap-southeast-2", "Sydney"),
        ("ap-northeast-1", "Tokyo"),
        ("ca-central-1", "Canada (Central)"),
        ("eu-central-1", "Frankfurt"),
        ("eu-west-1", "Ireland"),
        ("eu-west-2", "London"),
        ("eu-south-1", "Milan"),
        ("eu-west-3", "Paris"),
        ("eu-north-1", "Stockholm"),
        ("me-south-1", "Bahrain"),
        ("sa-east-1", "Sao Paulo"),
        ("us-east-2", "Ohio"),
        ("us-east-1", "N. Virginia"),
        ("us-west-1", "N. California"),
        ("us-west-2", "Oregon"),
    ];

    // FIXME from from 1.56
    #[allow(unused)]
    pub(super) fn region2region(region: &str) -> String {
        if let Some((_, r2)) = REGIONS.iter().find(|(r1, _)| *r1 == region) {
            return r2.to_string();
        }
        panic!("unknown region {}", region);
    }
}

pub(super) struct Printer {
    regions: Vec<Region>,
    instances: Vec<InstanceType>,
    price_changes: Vec<f64>,
    region_width: usize,
    instance_width: usize,
}

impl Printer {
    pub(super) fn new(regions: &[Region], instances: &[InstanceType], price_changes: &[f64]) -> Self {
        let is = instances.to_vec();
        Self {
            regions: regions.to_vec(),
            instances: is,
            price_changes: price_changes.to_vec(),
            region_width: Self::get_widest_region(regions),
            instance_width: Self::get_widest_instance(instances),
        }
    }

    fn print_line(&self) {
        println!(
            "|{}|",
            get_string_with_len_and_padding(
                "-",
                self.region_width + 2 + (self.instance_width + 3) * self.instances.len(),
                '-',
            )
        );
    }

    fn print_instances(&self, region: &Region) {
        for instance in &self.instances {
            if let Some(el) = region.find_instance(&*instance) {
                print!(" {} |", get_string_with_len(&el.to_string(), self.instance_width));
            } else {
                print!(" {} |", get_string_with_len("", self.instance_width));
            }
        }
    }

    fn print_instance_names(&self) {
        print!("| {} |", get_string_with_len("", self.region_width));
        for instance in &self.instances {
            print!(" {} |", get_string_with_len(instance.as_str(), self.instance_width));
        }
        println!();
    }

    fn print_regions(&self) {
        for region in &self.regions {
            if region.is_empty() {
                continue;
            }
            print!("| {} |", get_string_with_len(&region.get_region(), self.region_width));
            self.print_instances(region);
            println!();
        }
    }

    pub(super) fn print(&self) {
        self.print_line();

        self.print_instance_names(); // header

        self.print_line();

        self.print_regions(); // prices

        self.print_line();

        self.print_price_changes(); // relative price diffs

        self.print_line();
    }

    fn print_price_changes(&self) {
        print!("| {} |", get_string_with_len("", self.region_width));
        for change in &self.price_changes {
            print!(" {} |", get_f64_with_len(*change, self.instance_width))
        }
        print!(" {} |", get_string_with_len("", self.instance_width));
        println!();
    }

    fn get_widest_instance(instances: &[InstanceType]) -> usize {
        instances.iter().map(|el| el.as_str().len()).max().unwrap()
    }

    fn get_widest_region(data: &[Region]) -> usize {
        data.iter().map(|el| el.get_region().len()).max().unwrap()
    }
}
