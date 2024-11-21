use img::{GenericImage, GenericImageView, ImageBuffer, RgbImage};
use std::f64::consts::PI;
use rand::Rng;
use std::env;

fn lattice_centers(x: f64, y: f64, sz: f64) -> (f64, f64) {
    let h = sz / 2.0;
    let a = ((x - h) / sz) as isize;
    let b = a as f64 * sz + h;
    let c = ((y - h) / sz) as isize;
    let d = c as f64 * sz + h;
    (b, d)
}

fn hex_lattice_centers(x: f64, y: f64, sz: f64) -> (f64, f64) {
    let sqrt3 = 3.0f64.sqrt();
    
    // Размеры базового прямоугольника
    let hex_width = sz * 2.0;
    let hex_height = sz * sqrt3;
    
    // Преобразуем координаты в систему шестиугольной сетки
    let q = (2.0/3.0 * x) / sz;
    let r = (-1.0/3.0 * x + sqrt3/3.0 * y) / sz;
    let s = -q - r;  // cubic coordinates: q + r + s = 0
    
    // Округляем до ближайшего шестиугольника
    let q_round = q.round();
    let r_round = r.round();
    let s_round = s.round();
    
    let q_diff = (q_round - q).abs();
    let r_diff = (r_round - r).abs();
    let s_diff = (s_round - s).abs();
    
    // Корректируем округление
    let (q_final, r_final) = if q_diff > r_diff && q_diff > s_diff {
        (-r_round - s_round, r_round)
    } else if r_diff > s_diff {
        (q_round, -q_round - s_round)
    } else {
        (q_round, r_round)
    };
    
    // Преобразуем обратно в декартовы координаты
    let center_x = sz * 3.0/2.0 * q_final;
    let center_y = sz * sqrt3 * (r_final + q_final/2.0);
    
    (center_x, center_y)
}



fn osc(a: f64) -> f64 {
    a.sin() / a
}

fn osc2(a: f64, n: usize) -> f64 {
    let mut v = 0_f64;
    let x = a / n as f64;
    for i in 1..=n {
      v += (x * i as f64).sin();
    }
    v / n as f64
}

fn main() {
  let args: Vec<String> = env::args().collect();
    
    // Парсим аргументы командной строки
    let base = if args.len() > 1 {
        args[1].parse::<f64>().unwrap()
    } else {
        17.0
    };
    
    let multiplier = if args.len() > 2 {
        args[2].parse::<f64>().unwrap()
    } else {
        5.0
    };

    let pattern_scale = base / multiplier;
    println!("Using pattern scale: {}", pattern_scale);

    let mut rng = rand::thread_rng();
    let mut img = ImageBuffer::from_fn(1024, 1024, |x, y| {
        let r1: f64 = rng.gen();
        let r2: f64 = rng.gen();
        let (x2, y2) = lattice_centers(x as f64 - 512.0 + r1, y as f64 - 512.0 + r2, 3.0);
        let sqdist2 = x2*x2 + y2*y2;
        let v = (osc2(sqdist2 * PI / pattern_scale, 1) * 1.0 + 1.0) / 2.0;
        let w = (osc2(sqdist2 * PI / pattern_scale, 2) * 1.0 + 1.0) / 2.0;
        let u = (osc2(sqdist2 * PI / pattern_scale, 3) * 1.0 + 1.0) / 2.0;
        let a = ( v * 1.0 + w * 0.0 + u * 2.0 ) / ( 1.0 + 2.0 + 0.0 );
        let b = ( v * 1.0 + w * 1.0 + u * 1.0 ) / ( 1.0 + 1.0 + 1.0 );
        let c = ( v * 1.0 + w * 2.0 + u * 0.0 ) / ( 1.0 + 0.0 + 2.0 );
        let r: f64 = a.max(0.0).min(1.0) * 255.0;
        let g: f64 = b.max(0.0).min(1.0) * 255.0;
        let b: f64 = c.max(0.0).min(1.0) * 255.0;
        img::Rgb([r as u8,g as u8,b as u8])
    });
    img.save(format!("./tmp/img_sq_{base}_{multiplier}.png")).unwrap();
}
