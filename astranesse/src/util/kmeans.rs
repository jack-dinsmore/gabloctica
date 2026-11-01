use rand::Rng;
use rand_distr::{Distribution, Normal};


pub struct KMeans<const C: usize> {
    centroids: Vec<[f32; C]>,
    n: usize
}

impl<const C: usize> KMeans<C> {
    pub fn new(points: &[[f32; C]], n: usize, rng: &mut impl Rng) -> Self {
        let mut centroids = vec![[0.; C]; n];
        for i in 0..C {
            let mut m1 = 0.;
            let mut m2 = 0.;
            for p in points {
                m1 += p[i];
                m2 += p[i]*p[i];
            }
            let mean = m1 / points.len() as f32;
            let std = (m2 / points.len() as f32 - mean*mean).sqrt();
            let normal = Normal::new(mean, std).unwrap();
            for samp in 0..n {
                centroids[samp][i] = normal.sample(rng);
            }
        }

        // Perform algorithm
        for _ in 0..10 {
            let mut counts = vec![0; n];
            let mut means = vec![[0.; C]; n];
            for p in points {
                let mut best_distance2 = f32::INFINITY;
                let mut best_index = 0;
                for samp in 0..n {
                    let mut dist2 = 0.;
                    for i in 0..C {
                        dist2 += (p[i] - centroids[samp][i]).powi(2);
                    }

                    if dist2 < best_distance2 {
                        best_distance2 = dist2;
                        best_index = samp;
                    }
                }

                counts[best_index] += 1;
                for i in 0..C {
                    means[best_index][i] += p[i]
                }
            }

            for samp in 0..n {
                if counts[samp] == 0 {continue;}
                for i in 0..C {
                    centroids[samp][i] = means[samp][i] / counts[samp] as f32;
                }
            }
        }
        
        Self {
            n,
            centroids
        }
    }

    pub fn evaluate(&self, point: [f32; C]) -> usize {
        let mut best_distance2 = f32::INFINITY;
        let mut best_index = 0;
        for samp in 0..self.n {
            let mut dist2 = 0.;
            for i in 0..C {
                dist2 += (point[i] - self.centroids[samp][i]).powi(2);
            }

            if dist2 < best_distance2 {
                best_distance2 = dist2;
                best_index = samp;
            }
        }

        best_index
    }
}