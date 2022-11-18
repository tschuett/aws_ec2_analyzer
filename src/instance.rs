use crate::availability_zone::AvailabilityZone;
use aws_sdk_ec2::model::InstanceType;

pub(super) struct Instance {
    _region: String,
    _instance: InstanceType,
    spot_prices: Option<MinMax>,
    ondemand_price: f64,
}

impl Instance {
    pub(super) fn new(region: &str, _instance: InstanceType, zones: &[AvailabilityZone], ondemand: f64) -> Self {
        let data = Self::process_zones(zones);
        Self {
            _region: region.to_string(),
            _instance,
            spot_prices: data,
            ondemand_price: ondemand,
        }
    }

    fn process_zones(zones: &[AvailabilityZone]) -> Option<MinMax> {
        if zones.is_empty() {
            return None;
        }

        assert!(!zones.is_empty());

        let min_entry = zones
            .iter()
            .map(|e| e.get_min())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let max_entry = zones
            .iter()
            .map(|e| e.get_max())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let last_entry = zones
            .iter()
            .map(|e| e.get_last())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let sum_avg: f64 = zones.iter().map(|e| e.get_average()).sum();

        Some(MinMax {
            min: min_entry,
            avg: sum_avg / zones.len() as f64,
            max: max_entry,
            last: last_entry,
        })
    }

    //    #[allow(unused)]
    //    pub(super) fn get_spread(&self) -> f64 {
    //        self.max_price / self.min_price
    //    }

    pub(super) fn get_average_price(&self) -> Option<f64> {
        if let Some(price) = &self.spot_prices {
            return Some(price.avg);
        }
        None
    }
    //
    //#[allow(unused)]
    //pub(super) fn get_last_price(&self) -> f64 {
    //    self.last_price
    //}
    //
    pub(super) fn get_ondemand_price(&self) -> f64 {
        self.ondemand_price
    }
}

use std::fmt;

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            //            .field("x", &self.x)
            //            .field("y", &self.y)
            .finish()
    }
}

//impl Copy for Instance {}

impl Clone for Instance {
    fn clone(&self) -> Self {
        if let Some(spot_price) = &self.spot_prices {
            Instance {
                _region: self._region.clone(),
                _instance: self._instance.clone(),
                spot_prices: Some(MinMax {
                    min: spot_price.min,
                    avg: spot_price.avg,
                    max: spot_price.max,
                    last: spot_price.last,
                }),
                ondemand_price: self.ondemand_price,
            }
        } else {
            Instance {
                _region: self._region.clone(),
                _instance: self._instance.clone(),
                spot_prices: None,
                ondemand_price: self.ondemand_price,
            }
        }
    }
}

struct MinMax {
    min: f64,
    avg: f64,
    max: f64,
    last: f64,
}
