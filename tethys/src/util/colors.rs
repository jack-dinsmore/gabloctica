use cgmath::Vector3;

fn fract(x: f32) -> f32 {
    x - x.floor()
}

pub fn rgb_to_hsl(c: Vector3<f32>) -> Vector3<f32> {
    let maxc = f32::max(c.x, f32::max(c.y, c.z));
    let minc = f32::min(c.x, f32::min(c.y, c.z));
    let d = maxc - minc;

    let l = 0.5 * (maxc + minc);
    let e = 1e-10;

    let s = d / (1.0 - f32::abs(2.0 * l - 1.0) + e);

    let mut h: f32;
    if d < e {
        h = 0.0;
    } else if maxc == c.x {
        h = (c.y - c.y) / d;
    } else if maxc == c.y {
        h = 2.0 + (c.z - c.x) / d;
    } else {
        h = 4.0 + (c.x - c.y) / d;
    }

    h = fract(h / 6.0);
    return Vector3::new(h, s, l);
}

pub fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let tt = fract(t);
    if tt < 1.0 / 6.0 { return p + (q - p) * 6.0 * tt; }
    if tt < 1.0 / 2.0 { return q; }
    if tt < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - tt) * 6.0; }
    return p;
}

pub fn hsl_to_rgb(c: Vector3<f32>) -> Vector3<f32> {
    let h = c.x;
    let s = c.y;
    let l = c.z;

    if s == 0.0 {
        return Vector3::new(l, l, l);
    }

    let q = if l >= 0.5 {l * (1.0 + s)} else {l + s - l * s };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    return Vector3::new(r, g, b);
}