use crate::get_f64_with_len;
use crate::get_option_f64_with_len;
use crate::get_string_with_len;
use crate::get_string_with_len_and_padding;
use crate::print_spot_region::spot_region::SpotRegion;

pub(crate) struct Printer {
    regions: Vec<SpotRegion>,
    instances: Vec<String>,
    price_changes: Vec<f64>,
    region_width: usize,
    instance_width: usize,
    favorite_regions: Vec<String>,
}

impl Printer {
    pub(super) fn new(
        regions: &[SpotRegion],
        instances: &[String],
        price_changes: &[f64],
        favorite_regions: &[&str],
    ) -> Self {
        let is = instances.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let fav_regions = favorite_regions
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Self {
            regions: regions.to_vec(),
            instances: is,
            price_changes: price_changes.to_vec(),
            region_width: Self::get_widest_region(regions),
            instance_width: Self::get_widest_instance(instances),
            favorite_regions: fav_regions,
        }
    }

    pub(super) fn print(&self) {
        let fav_regions: Vec<SpotRegion> = self
            .regions
            .iter()
            .filter(|r| self.favorite_regions.contains(&r.get_region().to_string()))
            .cloned()
            .collect::<Vec<_>>();

        self.print_line();

        self.print_regions(&fav_regions); // prices

        self.print_line();

        self.print_instance_names(); // header

        self.print_line();

        self.print_regions(&self.regions); // prices

        self.print_line();

        self.print_price_changes(); // relative price diffs

        self.print_line();
    }

    fn get_widest_instance(instances: &[String]) -> usize {
        instances.iter().map(|el| el.len()).max().unwrap()
    }

    fn get_widest_region(data: &[SpotRegion]) -> usize {
        data.iter().map(|el| el.get_region().len()).max().unwrap()
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

    fn print_instance_names(&self) {
        print!("| {} |", get_string_with_len("", self.region_width));
        for instance in &self.instances {
            print!(" {} |", get_string_with_len(instance, self.instance_width));
        }
        println!();
    }

    fn print_regions(&self, regions: &[SpotRegion]) {
        for region in regions {
            if region.is_empty() {
                continue;
            }
            print!(
                "| {} |",
                get_string_with_len(&region.get_region(), self.region_width)
            );
            self.print_instances(region);
            println!();
        }
    }

    fn print_instances(&self, region: &SpotRegion) {
        for instance in &self.instances {
            if let Some(el) = region.find_instance(instance) {
                print!(
                    "  {} / {}  |",
                    get_option_f64_with_len(el.get_average_price(), self.instance_width / 2 - 2),
                    get_f64_with_len(el.get_ondemand_price(), self.instance_width / 2 - 2)
                );
            } else {
                // no instance found
                print!(" {} |", get_string_with_len("", self.instance_width));
            }
        }
    }

    fn print_price_changes(&self) {
        print!("| {} |", get_string_with_len("", self.region_width));
        for change in &self.price_changes {
            print!("  {} |", get_f64_with_len(*change, self.instance_width - 1))
        }
        print!(" {} |", get_string_with_len("", self.instance_width));
        println!();
    }
}
