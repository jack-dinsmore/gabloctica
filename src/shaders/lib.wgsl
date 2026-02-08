struct CameraUniform {
    view_proj: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};

fn rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    let maxc = max(c.r, max(c.g, c.b));
    let minc = min(c.r, min(c.g, c.b));
    let d = maxc - minc;

    let l = 0.5 * (maxc + minc);
    let e = 1e-10;

    let s = d / (1.0 - abs(2.0 * l - 1.0) + e);

    var h: f32;
    if (d < e) {
        h = 0.0;
    } else if (maxc == c.r) {
        h = (c.g - c.b) / d;
    } else if (maxc == c.g) {
        h = 2.0 + (c.b - c.r) / d;
    } else {
        h = 4.0 + (c.r - c.g) / d;
    }

    h = fract(h / 6.0);
    return vec3<f32>(h, s, l);
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let tt = fract(t);
    if (tt < 1.0 / 6.0) { return p + (q - p) * 6.0 * tt; }
    if (tt < 1.0 / 2.0) { return q; }
    if (tt < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - tt) * 6.0; }
    return p;
}

fn hsl_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let h = c.x;
    let s = c.y;
    let l = c.z;

    if (s == 0.0) {
        return vec3<f32>(l);
    }

    let q = select(
        l * (1.0 + s),
        l + s - l * s,
        l >= 0.5
    );
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    return vec3<f32>(r, g, b);
}


fn clamp_color(c: vec4<f32>) -> vec4<f32> {
    var hsl = rgb_to_hsl(c.xyz);
    hsl = round(hsl*32.) / 32.;
    hsl.y = 1.;
    return vec4(hsl_to_rgb(hsl), c.w);
}