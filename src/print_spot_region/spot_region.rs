use crate::instance::Instance;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct SpotRegion {
    region: String,
    prices: HashMap<String, Instance>,
}

impl SpotRegion {
    pub(super) fn new(region: &str) -> Self {
        Self {
            region: region.to_string(),
            prices: HashMap::default(),
        }
    }

    pub(super) fn add(&mut self, instance: &str, data: Instance) {
        self.prices.insert(instance.to_string(), data);
    }

    pub(super) fn find_instance(&self, instance: &str) -> Option<&Instance> {
        self.prices.get(instance)
    }

    pub(super) fn contains(&self, instance: &str) -> bool {
        self.prices.contains_key(instance)
    }

    pub(super) fn get_region(&self) -> Cow<'static, str> {
        Cow::from(self.region.clone())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.prices.is_empty()
    }
}
