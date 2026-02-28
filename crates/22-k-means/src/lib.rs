// ============================================================
//  YOUR CHALLENGE — implement k-means clustering (Lloyd's algorithm).
//
//  Algorithm:
//    1. Initialise k centroids by picking k random data points
//    2. Assign every point to its nearest centroid (Euclidean)
//    3. Recompute each centroid as the mean of its assigned points
//    4. Repeat 2-3 until centroids stop moving (or max_iter reached)
//
//  Used in: customer segmentation, log anomaly detection,
//           network traffic clustering, image colour quantisation.
// ============================================================

use rand::Rng;

/// Euclidean distance between two equal-length vectors.
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

pub struct KMeans {
    pub k:        usize,
    pub max_iter: usize,
    pub centroids: Vec<Vec<f64>>,
}

impl KMeans {
    pub fn new(k: usize, max_iter: usize) -> Self {
        Self { k, max_iter, centroids: Vec::new() }
    }

    /// Fit centroids to `data`.  Returns the number of iterations run.
    pub fn fit(&mut self, data: &[Vec<f64>], rng: &mut impl Rng) -> usize {
        assert!(!data.is_empty(), "data must not be empty");
        let k = self.k.min(data.len());

        // Initialise centroids by sampling k distinct data points (without replacement).
        let mut indices: Vec<usize> = (0..data.len()).collect();
        for i in 0..k {
            let j = rng.gen_range(i..data.len());
            indices.swap(i, j);
        }
        self.centroids = indices[..k].iter().map(|&i| data[i].clone()).collect();

        let mut iters = 0;
        for _ in 0..self.max_iter {
            iters += 1;
            let assignments: Vec<usize> = data.iter().map(|p| self.predict(p)).collect();

            // Recompute centroids as the mean of assigned points.
            let mut new_centroids: Vec<Vec<f64>> = vec![vec![0.0; data[0].len()]; k];
            let mut counts = vec![0usize; k];
            for (point, &cluster) in data.iter().zip(assignments.iter()) {
                for (c, &p) in new_centroids[cluster].iter_mut().zip(point.iter()) {
                    *c += p;
                }
                counts[cluster] += 1;
            }
            for (c, &count) in new_centroids.iter_mut().zip(counts.iter()) {
                if count > 0 {
                    for val in c.iter_mut() { *val /= count as f64; }
                }
            }

            // Convergence check: centroids stopped moving.
            let converged = self.centroids.iter().zip(new_centroids.iter())
                .all(|(old, new)| euclidean_distance(old, new) < 1e-8);
            self.centroids = new_centroids;
            if converged { break; }
        }
        iters
    }

    /// Return the index of the nearest centroid to `point`.
    pub fn predict(&self, point: &[f64]) -> usize {
        self.centroids.iter().enumerate()
            .min_by(|(_, a), (_, b)| {
                euclidean_distance(a, point)
                    .partial_cmp(&euclidean_distance(b, point))
                    .unwrap()
            })
            .map(|(i, _)| i)
            .expect("centroids must not be empty — call fit() first")
    }

    /// Sum of squared distances from each point to its assigned centroid.
    pub fn inertia(&self, data: &[Vec<f64>]) -> f64 {
        data.iter()
            .map(|p| {
                let c = &self.centroids[self.predict(p)];
                euclidean_distance(p, c).powi(2)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    mod distances {
        use super::*;

        #[test]
        fn identical_points_have_zero_distance() {
            assert!((euclidean_distance(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0])).abs() < 1e-10);
        }

        #[test]
        fn distance_is_symmetric() {
            let a = vec![1.0, 2.0];
            let b = vec![4.0, 6.0];
            assert!((euclidean_distance(&a, &b) - euclidean_distance(&b, &a)).abs() < 1e-10);
        }
    }

    mod clustering {
        use super::*;

        #[test]
        fn k1_assigns_all_to_same_cluster() {
            let mut rng = StdRng::seed_from_u64(0);
            let data: Vec<Vec<f64>> = (0..10).map(|i| vec![i as f64, 0.0]).collect();
            let mut km = KMeans::new(1, 50);
            km.fit(&data, &mut rng);
            let labels: Vec<usize> = data.iter().map(|p| km.predict(p)).collect();
            assert!(labels.iter().all(|&l| l == 0), "k=1 → all same cluster");
        }

        #[test]
        fn three_well_separated_clusters_are_found() {
            let mut rng = StdRng::seed_from_u64(42);
            // Clusters centred at (0,0), (100,0), (50,100) — far apart
            let mut data: Vec<Vec<f64>> = Vec::new();
            for i in 0..10 { data.push(vec![i as f64 * 0.1,    i as f64 * 0.1]); }   // near (0,0)
            for i in 0..10 { data.push(vec![100.0 + i as f64 * 0.1, i as f64 * 0.1]); } // near (100,0)
            for i in 0..10 { data.push(vec![50.0 + i as f64 * 0.1, 100.0 + i as f64 * 0.1]); } // near (50,100)

            let mut km = KMeans::new(3, 100);
            km.fit(&data, &mut rng);

            // All points in the same original group should share a cluster label
            let g1: Vec<usize> = data[0..10].iter().map(|p| km.predict(p)).collect();
            let g2: Vec<usize> = data[10..20].iter().map(|p| km.predict(p)).collect();
            let g3: Vec<usize> = data[20..30].iter().map(|p| km.predict(p)).collect();

            assert!(g1.iter().all(|&l| l == g1[0]), "group 1 should be in same cluster");
            assert!(g2.iter().all(|&l| l == g2[0]), "group 2 should be in same cluster");
            assert!(g3.iter().all(|&l| l == g3[0]), "group 3 should be in same cluster");
            assert!(g1[0] != g2[0] && g2[0] != g3[0] && g1[0] != g3[0],
                "each group should be in a different cluster");
        }

        #[test]
        fn inertia_is_non_negative() {
            let mut rng = StdRng::seed_from_u64(1);
            let data: Vec<Vec<f64>> = (0..20).map(|i| vec![i as f64]).collect();
            let mut km = KMeans::new(4, 50);
            km.fit(&data, &mut rng);
            assert!(km.inertia(&data) >= 0.0);
        }

        #[test]
        fn predict_assigns_nearest_centroid() {
            let mut rng = StdRng::seed_from_u64(7);
            let data = vec![vec![0.0], vec![1.0], vec![10.0], vec![11.0]];
            let mut km = KMeans::new(2, 50);
            km.fit(&data, &mut rng);
            // 0 and 1 should be in the same cluster; 10 and 11 in the other
            assert_eq!(km.predict(&[0.0]), km.predict(&[1.0]));
            assert_eq!(km.predict(&[10.0]), km.predict(&[11.0]));
            assert_ne!(km.predict(&[0.0]), km.predict(&[10.0]));
        }

        #[test]
        fn centroids_lie_within_data_range() {
            let mut rng = StdRng::seed_from_u64(3);
            let data: Vec<Vec<f64>> = (0..20).map(|i| vec![i as f64, (i * 2) as f64]).collect();
            let mut km = KMeans::new(3, 50);
            km.fit(&data, &mut rng);
            for c in &km.centroids {
                assert!(c[0] >= 0.0 && c[0] <= 19.0, "centroid x out of range: {}", c[0]);
                assert!(c[1] >= 0.0 && c[1] <= 38.0, "centroid y out of range: {}", c[1]);
            }
        }
    }
}
