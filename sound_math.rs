fn smooth(t: f64, delay: f64, phase: f64) -> f64 {
    let p = phase * PI;
    let a: f64 = (t / delay + p).tan().atan() * delay;
    let b = (a - t/2_f64)/(PI*2_f64);
    a.tanh() - b
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
