
use crate::{game::planet::Planet, graphics::{Camera, Graphics, Renderer, ResourceType, Uniform, UniformBuffer}};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    viewport: [f32; 4],
    fog: [f32; 4],
    normal: [f32; 4],
}

impl Uniform for PostUniform {
    const TYPE: ResourceType = ResourceType::Post;
}

pub struct PostInfo {
    uniform: PostUniform,
    buffer: UniformBuffer<PostUniform>,
}
impl PostInfo {
    pub fn new(graphics: &Graphics, camera: &Camera) -> Self {
        let buffer = UniformBuffer::new(graphics);
        let size = graphics.window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let uniform = PostUniform {
            viewport: [width as f32, height as f32, camera.znear, camera.zfar],
            fog: [0.6, 0.09, 0., 1.],
            normal: [0., 0., 0., 0.],
        };

        Self {
            buffer,
            uniform,
        }
    }
    pub fn bind(&mut self, renderer: &mut Renderer) {
        self.buffer.bind(renderer);
    }
    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &Camera, my_planet: Option<&Planet>) {
        let scale_height = 3.;

        match my_planet {
            Some(planet) => {
                let planet_object = planet.object.as_ref().unwrap().borrow();
                let displacement = camera.pos - planet_object.body.pos.cast().unwrap();
        
                let altitude = displacement.x.abs().max(displacement.y.abs().max(displacement.z.abs()));
                let normal = if altitude == displacement.x {[1., 0., 0., 0.]}
                else if altitude == -displacement.x {[-1., 0., 0., 0.]}
                else if altitude == displacement.y {[0., 1., 0., 0.]}
                else if altitude == -displacement.y {[0., -1., 0., 0.]}
                else if altitude == displacement.z {[0., 0., 1., 0.]}
                else {[0., 0., -1., 0.]};
                let altitude = altitude as f32 - (planet.width/2) as f32;
                self.uniform.fog[3] = (-altitude / scale_height).min(0.).exp();
                self.uniform.normal = normal;
            },
            None => {
                self.uniform.fog[3] = 0.;
                self.uniform.normal = [0., 0., 1., 1.];
            },
        }
        self.buffer.write(graphics, self.uniform);
    }
}