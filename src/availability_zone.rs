pub(super) struct AvailabilityZone {
    min: f64,
    avg: f64,
    max: f64,
    last: f64,
}

impl AvailabilityZone {
    pub(super) fn new(min: f64, avg: f64, max: f64, last: f64) -> Self {
        Self { min, avg, max, last }
    }

    pub(super) fn get_min(&self) -> f64 {
        self.min
    }

    pub(super) fn get_max(&self) -> f64 {
        self.max
    }

    pub(super) fn get_average(&self) -> f64 {
        self.avg
    }

    pub(super) fn get_last(&self) -> f64 {
        self.last
    }
}
