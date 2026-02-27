use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum StatsError {
    #[error("input data is empty")]
    Empty,
    #[error("percentile must be in [0.0, 1.0], got {0}")]
    InvalidPercentile(f64),
    #[error("insufficient data: need at least {needed} points, got {got}")]
    InsufficientData { needed: usize, got: usize },
}

pub fn mean(data: &[f64]) -> Result<f64, StatsError> {
    if data.is_empty() { return Err(StatsError::Empty); }
    Ok(data.iter().sum::<f64>() / data.len() as f64)
}

/// Population variance (divides by N, not N-1)
pub fn variance(data: &[f64]) -> Result<f64, StatsError> {
    let m = mean(data)?;
    Ok(data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64)
}

/// Sample variance (divides by N-1, unbiased estimator)
pub fn sample_variance(data: &[f64]) -> Result<f64, StatsError> {
    if data.len() < 2 { return Err(StatsError::InsufficientData { needed: 2, got: data.len() }); }
    let m = mean(data)?;
    Ok(data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64)
}

pub fn std_dev(data: &[f64]) -> Result<f64, StatsError> {
    Ok(variance(data)?.sqrt())
}

pub fn median(data: &[f64]) -> Result<f64, StatsError> {
    if data.is_empty() { return Err(StatsError::Empty); }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    if n % 2 == 1 {
        Ok(sorted[n / 2])
    } else {
        Ok((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    }
}

/// p in [0.0, 1.0]; uses linear interpolation (same as NumPy default)
pub fn percentile(data: &[f64], p: f64) -> Result<f64, StatsError> {
    if data.is_empty() { return Err(StatsError::Empty); }
    if !(0.0..=1.0).contains(&p) { return Err(StatsError::InvalidPercentile(p)); }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = p * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    Ok(sorted[lo] * (1.0 - frac) + sorted[hi] * frac)
}

/// Excess kurtosis (0 for normal distribution)
pub fn kurtosis(data: &[f64]) -> Result<f64, StatsError> {
    if data.len() < 4 { return Err(StatsError::InsufficientData { needed: 4, got: data.len() }); }
    let m = mean(data)?;
    let s = std_dev(data)?;
    if s == 0.0 { return Err(StatsError::InsufficientData { needed: 2, got: 1 }); }
    let n = data.len() as f64;
    let kurt = data.iter().map(|x| ((x - m) / s).powi(4)).sum::<f64>() / n;
    Ok(kurt - 3.0) // excess kurtosis
}

/// Pearson's moment skewness
pub fn skewness(data: &[f64]) -> Result<f64, StatsError> {
    if data.len() < 3 { return Err(StatsError::InsufficientData { needed: 3, got: data.len() }); }
    let m = mean(data)?;
    let s = std_dev(data)?;
    if s == 0.0 { return Ok(0.0); }
    let n = data.len() as f64;
    Ok(data.iter().map(|x| ((x - m) / s).powi(3)).sum::<f64>() / n)
}

/// Z-score normalisation: (x - mean) / std_dev
pub fn z_scores(data: &[f64]) -> Result<Vec<f64>, StatsError> {
    let m = mean(data)?;
    let s = std_dev(data)?;
    if s == 0.0 { return Ok(vec![0.0; data.len()]); }
    Ok(data.iter().map(|x| (x - m) / s).collect())
}

/// IQR-based outlier detection. Returns the values flagged as outliers (|z| > 1.5·IQR).
pub fn iqr_outliers(data: &[f64]) -> Result<Vec<f64>, StatsError> {
    let q1 = percentile(data, 0.25)?;
    let q3 = percentile(data, 0.75)?;
    let iqr = q3 - q1;
    let lo = q1 - 1.5 * iqr;
    let hi = q3 + 1.5 * iqr;
    Ok(data.iter().copied().filter(|&x| x < lo || x > hi).collect())
}

/// Summary statistics bundle
pub struct Summary {
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub p5: f64,
    pub p95: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub n_outliers: usize,
}

pub fn summarise(data: &[f64]) -> Result<Summary, StatsError> {
    Ok(Summary {
        mean:      mean(data)?,
        std_dev:   std_dev(data)?,
        median:    median(data)?,
        p5:        percentile(data, 0.05)?,
        p95:       percentile(data, 0.95)?,
        skewness:  skewness(data)?,
        kurtosis:  kurtosis(data)?,
        n_outliers: iqr_outliers(data)?.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_known_sequence() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(), 3.0);
    }

    #[test]
    fn mean_of_empty_is_error() {
        assert_eq!(mean(&[]).unwrap_err(), StatsError::Empty);
    }

    #[test]
    fn variance_of_constant_is_zero() {
        assert!(variance(&[5.0, 5.0, 5.0, 5.0]).unwrap() < 1e-10);
    }

    #[test]
    fn median_even_length_interpolates() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]).unwrap(), 2.5);
    }

    #[test]
    fn median_odd_length_picks_middle() {
        assert_eq!(median(&[3.0, 1.0, 2.0]).unwrap(), 2.0);
    }

    #[test]
    fn percentile_0_is_min_percentile_1_is_max() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 0.0).unwrap(), 1.0);
        assert_eq!(percentile(&data, 1.0).unwrap(), 5.0);
    }

    #[test]
    fn skewness_of_symmetric_data_is_near_zero() {
        let data: Vec<f64> = (-50..=50).map(|x| x as f64).collect();
        assert!(skewness(&data).unwrap().abs() < 1e-10);
    }

    #[test]
    fn z_scores_have_zero_mean_unit_variance() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let z = z_scores(&data).unwrap();
        let zm = z.iter().sum::<f64>() / z.len() as f64;
        let zv = z.iter().map(|x| x.powi(2)).sum::<f64>() / z.len() as f64;
        assert!(zm.abs() < 1e-10);
        assert!((zv - 1.0).abs() < 1e-10);
    }

    #[test]
    fn iqr_outliers_detects_spike() {
        let mut data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        data.push(9999.0);
        let outliers = iqr_outliers(&data).unwrap();
        assert!(outliers.contains(&9999.0));
    }

    #[test]
    fn sample_variance_larger_than_population_variance() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let pop = variance(&data).unwrap();
        let samp = sample_variance(&data).unwrap();
        assert!(samp > pop);
    }
}
