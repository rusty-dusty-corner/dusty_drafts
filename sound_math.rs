fn smooth(t: f64, delay: f64, phase: f64, short: f64) -> f64 {
    let p = phase * PI;
    let a: f64 = (t / delay + p).tan().atan() * delay;
    let b = (a - t)/(PI*delay)*2.0;
    ((a * short).tanh() - b) / short
}

fn sharps(sm: f64, t: f64) -> f64 {
   let mut y = 0_f64;

   y += sharp(sm, t * (15.0 / 40.0)) / (15.0 / 40.0);
   y += sharp(sm, t * (30.0 / 40.0)) / (30.0 / 40.0);
   y += sharp(sm, t * (36.0 / 40.0)) / (36.0 / 40.0);
   y += sharp(sm, t * (40.0 / 40.0)) / (40.0 / 40.0);
   y += sharp(sm, t * (48.0 / 40.0)) / (48.0 / 40.0);
   y += sharp(sm, t * (60.0 / 40.0)) / (60.0 / 40.0);
   y += sharp(sm, t * (120.0 / 40.0)) / (120.0 / 40.0);

   //y += sharp(sm, t * (12.0 / 48.0)) / (12.0 / 48.0) * 12.0;
   //y += sharp(sm, t * (24.0 / 48.0)) / (24.0 / 48.0) * 24.0;
   //y += sharp(sm, t * (48.0 / 48.0)) / (48.0 / 48.0) * 48.0;

   //y += sharp(t * (30.0 / 48.0)) / (30.0 / 48.0) * 240.0;
   //y += sharp(t * (36.0 / 48.0)) / (36.0 / 48.0) * 144.0;

   //y += sharp(t * (40.0 / 48.0)) / (40.0 / 48.0) * 40.0;
   //y += sharp(sm, t * (4.0 / 48.0)) / (4.0 / 48.0) * 40.0;

   y / 7.0
}

#[inline(always)]
fn nrm(t: f64) -> f64 {
   (-t*t).exp()
}

#[inline(always)]
fn env(t: f64, period: f64, short: f64) -> f64 {
   let m1 = PI / period;
   let t2 = (t * m1).tan().atan() / m1 * short;
   nrm((-t2).exp()*3_f64 - 3_f64)
}

#[inline(always)]
fn sharp(sm: f64, t: f64) -> f64 {
   (t.sin().asin()/PI*2_f64*sm).atanh() / 2_f64.sqrt()
}

#[inline(always)]
fn tri(t: f64) -> f64 {
   let smooth = 0.9_f64;
   (t.sin() * smooth).asin() / smooth
}

#[inline(always)]
fn sub01(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    let x = ln2(a + 0.001) * PI - ln2(b) * PI;
    ( 1.0 - rabbit( x ) ).powf(c) / (x*x+d).sqrt().powf(e)
}


#[inline(always)]
fn sinc(t: f64) -> f64 {
    if t == 0_f64 { return 1_f64 }
    t.sin() / t
}

#[inline(always)]
fn trim(x: f64) -> f64 {
    x.max(0_f64)
}

#[inline(always)]
fn ln2(x: f64) -> f64 {
    x.log(2_f64)
}

#[inline(always)]
fn rabbit(x: f64) -> f64 {
    let a = (x.cos() * 0.999_f64).asin()/PI*2_f64;
    (0.999_f64 - a*a).sqrt()
}

#[inline(always)]
fn sinc_osc(t: f64, impulse_freq: f64, sinc_freq: f64) -> f64 {
    let m1 = impulse_freq * PI;
    let t2 = tri(t * m1) / m1;
    let m2 = sinc_freq * 2.0 * PI;
    sinc(t2 * m2)
}

#[inline(always)]
fn sinc_osc_plus(t: f64, impulse_freq: f64, sinc_freq: f64, a: f64, b: f64) -> f64 {
    let m1 = impulse_freq * PI;
    let t2 = tri(t * m1 + a) / m1;
    let m2 = sinc_freq * 2.0 * PI;
    sinc(t2 * m2 + b)
}

fn main() {
    use rustfft::num_traits::FloatConst;
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut rng = rand::thread_rng();
}

#[inline(always)]
fn osc(t: f64) -> f64 {
    let sample = (t * 440.0 * 2.0 * PI).sin();
    sample
}
