use crate::game::object::Planet;
use crate::graphics::{Camera, Graphics, Lighting, NEAR_DISTANCE, PostRenderer, PostVertex, RENDER_DISTANCE, RenderTarget, Renderer, ResourceType, Shader};
use crate::graphics::{UniformBuffer, Uniform};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    planet_model_inv: [[f32; 4]; 4],
    viewport: [f32; 4],
    fog: [f32; 4],
}
impl Uniform for PostUniform {
    const TYPE: ResourceType = ResourceType::Post;
}


pub struct PostProcessor {
    shader: Shader,
    post: PostRenderer,
    buffer: UniformBuffer<PostUniform>,
    uniform: PostUniform,
}

impl PostProcessor {
    pub fn new(graphics: &mut Graphics, camera: &Camera) -> Self {
        let shader = Shader::new::<PostVertex>(graphics, include_str!("../../shaders/post.wgsl"), vec![
            ResourceType::Post,
            ResourceType::Camera,
            ResourceType::Lighting,
        ], RenderTarget::Screen);

        let size = graphics.window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let uniform = PostUniform {
            viewport: [width as f32, height as f32, NEAR_DISTANCE, RENDER_DISTANCE],
            fog: [0.6, 0.09, 0., 1.],
            planet_model_inv: [[0., 0., 0., 0.],[0., 0., 0., 0.],[0., 0., 0., 0.],[0., 0., 0., 0.]],
        };
        let buffer = UniformBuffer::new(graphics);

        let post = PostRenderer::new(graphics, &buffer);

        Self {
            shader,
            uniform,
            buffer,
            post
        }
    }

    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &Camera, my_planet: Option<&Planet>) {
        match my_planet {
            Some(p) => match p.get_post_params(camera) {
                Some((planet_model_inv, fog)) => {
                    self.uniform.planet_model_inv = planet_model_inv;
                    self.uniform.fog = fog;
                }
                None => self.uniform.fog[3] = 0.
            },
            None => self.uniform.fog[3] = 0.
        }
        self.buffer.write(graphics, self.uniform);
    }

    pub fn render(&self, renderer: &mut Renderer, camera: &Camera, lighting: &Lighting) {
        renderer.start_post();
        self.shader.bind(renderer);
        camera.bind(renderer);
        lighting.bind(renderer);
        self.post.bind(renderer);
        renderer.stop_post();
    }
}