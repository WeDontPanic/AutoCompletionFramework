use self::item::EngineItem;

pub mod item;

/// Collection of different types of weights for a relevance calculation
#[derive(Clone, Copy, Debug)]
pub struct RelevanceWeights {
    pub str_weight: f64,
    pub freq_weight: f64,
    pub total_weight: f64,
}

impl RelevanceWeights {
    #[inline]
    pub fn new(str_weight: f64, freq_weight: f64, total_weight: f64) -> Self {
        Self {
            str_weight,
            freq_weight,
            total_weight,
        }
    }
}

impl Default for RelevanceWeights {
    #[inline]
    fn default() -> Self {
        Self {
            str_weight: 1.0,
            freq_weight: 1.0,
            total_weight: 1.0,
        }
    }
}

/// Calculates the relevance for EngineItems
#[derive(Clone, Copy, Debug)]
pub struct RelevanceCalc {
    weights: RelevanceWeights,
}

impl RelevanceCalc {
    /// Create a new RelevanceCalc
    #[inline]
    pub fn new(weights: RelevanceWeights) -> Self {
        Self { weights }
    }

    #[inline]
    pub fn with_total_weight(mut self, total_weight: f64) -> Self {
        self.weights.total_weight = total_weight;
        self
    }

    /// Executes relevance calculation for a given Item
    #[inline]
    pub fn calc(&self, item: &EngineItem, str_rel: u16) -> u16 {
        let srel = ((str_rel as f64) * self.weights.str_weight).min(1000.0);
        let mut frel =
            ((item.inner().frequency() * 1000000.0) * self.weights.freq_weight).min(1000.0);
        // Give words with frequency info at least some value
        if frel > 0.0 && frel < 1.0 {
            frel = 1.0;
        }
        let calc = (srel + frel + 1.0) * self.weights.total_weight;
        (calc * 10.0) as u16
    }
}
