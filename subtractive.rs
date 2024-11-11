use hound;
use rand::prelude::*;
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::f64::consts::PI;
use std::i16;
use std::sync::Arc;

use super::SAMPLE_RATE;

type Pair<T> = (T, T);

pub struct Subtractive {
    pub fft: Arc<dyn Fft<f64>>,
    pub ifft: Arc<dyn Fft<f64>>,
    pub preset: Vec<Pair<[Complex<f64>; SAMPLE_RATE as usize]>>,
}

impl Subtractive {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(SAMPLE_RATE as usize);
        let ifft = planner.plan_fft_inverse(SAMPLE_RATE as usize);
        Self {
            fft,
            ifft,
            preset: vec![],
        }
    }

    pub fn new_preset<G, F>(&mut self, g: G, f: F)
    where
        G: Fn(&mut Complex<f64>, f64, f64),
        F: Fn(&mut Complex<f64>, f64, f64),
    {
        let mut rng = rand::thread_rng();
        let mut noise_a = [Complex::default(); SAMPLE_RATE as usize];
        let mut noise_b = [Complex::default(); SAMPLE_RATE as usize];
        for i in 0..SAMPLE_RATE as usize {
            noise_a[i].re = rng.gen();
            noise_b[i].re = rng.gen();
            let x = (i as f64 * 2.0 - 1.0) / SAMPLE_RATE as f64;
            g(&mut noise_a[i], 0.0, x);
            g(&mut noise_b[i], 1.0, x);
        }
        self.fft.process(&mut noise_a);
        self.fft.process(&mut noise_b);
        let norm = (SAMPLE_RATE as f64).sqrt();
        for i in 0..SAMPLE_RATE as usize {
            noise_a[i] /= norm;
            noise_b[i] /= norm;
            f(&mut noise_a[i], 0.0, i as f64);
            f(&mut noise_b[i], 1.0, i as f64);
        }
        self.ifft.process(&mut noise_a);
        self.ifft.process(&mut noise_b);
        for i in 0..SAMPLE_RATE as usize {
            noise_a[i] /= norm;
            noise_b[i] /= norm;
        }
        self.preset.push((noise_a, noise_b));
    }
}
