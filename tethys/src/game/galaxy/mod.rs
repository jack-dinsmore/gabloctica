use std::f32::consts::PI;

use cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector3};
use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, Perlin};
use rand::{RngExt, rngs::ThreadRng};
use rustc_hash::FxHashMap;

use crate::{graphics::{Camera, FlatVertex, Graphics, IndexBuffer, ModelUniform, Renderer, Texture, UniformBuffer, VertexBuffer}, util::percentile};

const MAX_STEPS: usize = 300;
const MAX_RADIUS: f32 = 1e8;
const NOISE_SCALE: f32 = 1e6;

pub struct Galaxy {
    dust_noise: Perlin,
    stellar_noise: Perlin,
    index_buffer: IndexBuffer,
    vertex_buffer: VertexBuffer<FlatVertex>,
    model_buffer: UniformBuffer<ModelUniform>,
    texture: Option<Texture>,
    star_h: f32,
    dust_h: f32,
    halflight_r: f32,
    stars: Vec<Star>
}
impl Galaxy {
    pub fn new(graphics: &Graphics) -> Self {
        let indices = vec![
            0,1,2,0,2,3,
            4,5,6,4,6,7,
            8,9,10,8,10,11,
            12,13,14,12,14,15,
            16,17,18,16,18,19,
            20,21,22,20,22,23,
        ];
        let vertices = vec![
            FlatVertex { x: [-1., -1., 1., 1.], texpos: [0., 0.] },
            FlatVertex { x: [-1., 1., 1., 1.], texpos: [0., 0.5] },
            FlatVertex { x: [1., 1., 1., 1.], texpos: [0.33333333, 0.5] },
            FlatVertex { x: [1., -1., 1., 1.], texpos: [0.33333333, 0.] },

            FlatVertex { x: [-1., -1., -1., 1.], texpos: [0.33333333, 0.] },
            FlatVertex { x: [1., -1., -1., 1.], texpos: [0.33333333, 0.5] },
            FlatVertex { x: [1., 1., -1., 1.], texpos: [0.66666667, 0.5] },
            FlatVertex { x: [-1., 1., -1., 1.], texpos: [0.66666667, 0.] },

            FlatVertex { x: [-1., 1., -1., 1.], texpos: [0.666666667, 0.0] },
            FlatVertex { x: [1., 1., -1., 1.], texpos: [0.666666667, 0.5] },
            FlatVertex { x: [1., 1., 1., 1.], texpos: [1., 0.5] },
            FlatVertex { x: [-1., 1., 1., 1.], texpos: [1., 0.] },

            FlatVertex { x: [-1., -1., -1., 1.], texpos: [0., 0.5] },
            FlatVertex { x: [-1., -1., 1., 1.], texpos: [0., 1.] },
            FlatVertex { x: [1., -1., 1., 1.], texpos: [0.333333333, 1.] },
            FlatVertex { x: [1., -1., -1., 1.], texpos: [0.333333333, 0.5] },

            FlatVertex { x: [1., -1., -1., 1.], texpos: [0.333333333, 0.5] },
            FlatVertex { x: [1., -1., 1., 1.], texpos: [0.333333333, 1.] },
            FlatVertex { x: [1., 1., 1., 1.], texpos: [0.666666667, 1.] },
            FlatVertex { x: [1., 1., -1., 1.], texpos: [0.666666667, 0.5] },

            FlatVertex { x: [-1., -1., -1., 1.], texpos: [0.666666667, 0.5] },
            FlatVertex { x: [-1., 1., -1., 1.], texpos: [0.666666667, 1.] },
            FlatVertex { x: [-1., 1., 1., 1.], texpos: [1., 1.] },
            FlatVertex { x: [-1., -1., 1., 1.], texpos: [1., 0.5] },
        ];
        let index_buffer = IndexBuffer::new(graphics, indices.len());
        index_buffer.write(graphics, indices);
        let vertex_buffer = VertexBuffer::new(graphics, vertices.len());
        vertex_buffer.write(graphics, vertices);
        let model_buffer = UniformBuffer::new(graphics);
        model_buffer.write(graphics, ModelUniform::new(Matrix4::identity()));

        let star_h = 8e6;
        let dust_h = 5e6;
        let halflight_r = 1e7;
        let max_rho = MAX_RADIUS / halflight_r;

        let mut stars = Vec::new();
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let theta: f32 = rng.random::<f32>() * 2. * PI;
            let mut z = rng.random::<f32>().ln() * star_h;
            if rng.random::<f32>() < 0.5 {
                z *= -1.;
            }
            let mut rho = ((1. + max_rho.powi(2)).powf(rng.random()) - 1.).sqrt();
            rho *= halflight_r;
            stars.push(Star::new(Vector3::new(rho*theta.cos(), rho*theta.sin(), z), &mut rng));
        }

        Self {
            dust_noise: Perlin::new(1525252),//TODO
            stellar_noise: Perlin::new(1525252),//TODO
            index_buffer,
            vertex_buffer,
            texture: None,
            model_buffer: model_buffer,
            star_h,
            dust_h,
            halflight_r,
            stars,
        }
    }

    pub fn update_skybox(&mut self, graphics: &Graphics, camera: &Camera, camera_pos: Vector3<f32>) {
        let size = 512;
        let mut image = ImageBuffer::from_pixel(3*size, 2*size, Rgba([0, 0, 0, 255]));

        // Partition the stars
        let mut star_map = FxHashMap::default();
        for (star_index, star) in self.stars.iter().enumerate() {
            let mut delta = star.pos - camera_pos;
            delta /= delta.x.abs().max(delta.y.abs().max(delta.z.abs()));
            let xi = ((delta.x + 1.) / 2. * size as f32) as u32;
            let yi = ((delta.y + 1.) / 2. * size as f32) as u32;
            let zi = ((delta.z + 1.) / 2. * size as f32) as u32;
            let (face, x, y) = if (delta.z - 1.).abs() < 1e-5 {
                (0, xi, yi)
            } else if (delta.z + 1.).abs() < 1e-5 {
                (1, yi, xi)
            } else if (delta.y - 1.).abs() < 1e-5 {
                (2, zi, xi)
            } else if (delta.y + 1.).abs() < 1e-5 {
                (3, xi, zi)
            } else if (delta.x - 1.).abs() < 1e-5 {
                (4, yi, zi)
            } else {
                (5, zi, yi)
            };
            let index = (face*size*size + x*size + y) as usize;
            if !star_map.contains_key(&index) {
                star_map.insert(index, Vec::new());
            }
            star_map.get_mut(&index).unwrap().push(star_index)
        }

        // Get colors
        let mut colors = Vec::new();
        for face in 0..6 {
            let (start_x, start_y) = match face {
                0 => (0, 0),
                1 => (size, 0),
                2 => (2*size, 0),
                3 => (0, size),
                4 => (size, size),
                5 => (2*size, size),
                _ => unreachable!()
            };
            for x in 0..size {
                let xf = 2. * x as f32 / size as f32 - 1.;
                for y in 0..size {
                    let yf = 2. * y as f32 / size as f32 - 1.;
                    let dir = match face {
                        0 => Vector3::new(xf, yf, 1.),
                        1 => Vector3::new(yf, xf, -1.),
                        2 => Vector3::new(yf, 1., xf),
                        3 => Vector3::new(xf, -1., yf),
                        4 => Vector3::new(1., xf, yf),
                        5 => Vector3::new(-1., yf, xf),
                        _ => unreachable!()
                    }.normalize();

                    let star_vec = star_map.get(&colors.len());
                    let color = self.raytrace(camera_pos, dir, star_vec);
                    colors.push(((start_x + x, start_y + y), color));
                }
            }
        }

        const BRIGHTNESS: f32 = 1.;
        let min_color = percentile(&colors.iter().map(|c| c.1[0].max(c.1[1].max(c.1[2]))).collect::<Vec<_>>(), 50.);
        let max_color = colors.iter().fold(0., |a, c| c.1[0].max(c.1[1].max(c.1[2].max(a))));
        for (pos, color) in colors {
            image[pos] = Rgba([
                ((color[0]-min_color)/(max_color-min_color) * BRIGHTNESS * 255.) as u8,
                ((color[1]-min_color)/(max_color-min_color) * BRIGHTNESS * 255.) as u8,
                ((color[2]-min_color)/(max_color-min_color) * BRIGHTNESS * 255.) as u8,
                255
            ]);
        }

        self.texture = Some(Texture::from_image(graphics, &camera, &image, (3*size, 2*size)));
    }

    fn raytrace(&self, camera: Vector3<f32>, dir: Vector3<f32>, star_indices: Option<&Vec<usize>>) -> [f32; 3] {
        let d_step = MAX_RADIUS / MAX_STEPS as f32;
        let mut color = [0., 0., 0.];

        let mut star_queries = Vec::new();
        if let Some(indices) = star_indices {
            for index in indices {
                let star = &self.stars[*index];
                let dist = (star.pos - camera).magnitude();
                let star_flux = 10. * star.lum / (dist*dist);
                star_queries.push((dist, star_flux));
            }
        }

        let mut dist = 0.;
        let mut step = d_step;
        while dist < MAX_RADIUS {
            dist += step;

            let pos = camera + dir*dist;
            if pos.z.abs() > self.dust_h*3. {continue;}
            let rad2 = pos.x*pos.x + pos.y*pos.y;
            let mut brightness = (-pos.z.abs()/self.star_h).exp();
            brightness *= 1. / (1. + rad2/(self.halflight_r*self.halflight_r));

            let mut dust = (-pos.z.abs()/self.dust_h).exp();

            let noise_pos = [
                (pos.x/NOISE_SCALE) as f64,
                (pos.y/NOISE_SCALE) as f64,
                (pos.z/NOISE_SCALE) as f64
            ];
            brightness *= (2. + self.stellar_noise.get(noise_pos)).abs() as f32;
            dust *= (1. + self.dust_noise.get(noise_pos)).powi(2) as f32;
            color[0] += brightness * 5e-9 * step;
            color[1] += brightness * 7e-9 * step;
            color[2] += brightness * 10e-9 * step;

            color[0] *= 1. - dust * 3e-9 * step;
            color[1] *= 1. - dust * 5e-9 * step;
            color[2] *= 1. - dust * 9e-9 * step;

            step = d_step / (brightness.max(dust).max(0.01));

            if !color[0].is_finite() {
                dbg!(brightness, step, dust, color[0]);
                panic!();
            }

            for (star_dist, flux) in &star_queries {
                if dist < *star_dist && dist + step > *star_dist {
                    color[0] += flux;
                    color[1] += flux;
                    color[2] += flux;
                }
            }
        }

        [color[0].ln(), color[1].ln(), color[2].ln()]
    }


    pub fn draw_skybox(&self, renderer: &mut Renderer) {
        self.vertex_buffer.bind(renderer);
        self.index_buffer.bind(renderer);
        self.model_buffer.bind(renderer);
        self.texture.as_ref().unwrap().bind(renderer);
        renderer.draw_indices(36);
    }
}

#[derive(Clone, Debug)]
struct Star {
    pos: Vector3<f32>,
    lum: f32,
}
impl Star {
    pub fn new(pos: Vector3<f32>, rng: &mut ThreadRng) -> Self {
        const MIN_LUM: f32 = 5e12;
        let lum = MIN_LUM * rng.random::<f32>().powf(-0.5);
        Self {
            pos,
            lum,
        }
    }
}