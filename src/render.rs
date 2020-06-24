use anyhow::Result;
use glsp::{bail, lib, rdata, rdata_impls, rfn, GResult, Runtime};
use lyon::{
    math::Point,
    path::PathEvent,
    tessellation::{
        geometry_builder::{FillVertexConstructor, StrokeVertexConstructor},
        BuffersBuilder, FillAttributes, FillOptions, FillTessellator, StrokeAttributes,
        VertexBuffers,
    },
};
use miniquad::{graphics::*, Context};
use std::mem;
use usvg::Color;

const MAX_MESH_INSTANCES: usize = 1024 * 1024;

rdata! {
/// A reference to an uploaded vector path.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mesh(usize);
}

lib! {
/// A wrapper around the OpenGL calls so the main file won't be polluted.
pub struct Render {
    /// The OpenGL pipeline for the pass rendering to the render target.
    pipeline: Pipeline,
    /// A list of draw calls with bindings that will be generated.
    draw_calls: Vec<DrawCall>,
    /// Whether some draw calls are missing bindings.
    missing_bindings: bool,

    camera_pan: (f32, f32),
    camera_zoom: f32,
}
}

impl Render {
    /// Setup the OpenGL pipeline and the texture for the framebuffer.
    pub fn new(ctx: &mut Context) -> Self {
        // Create an OpenGL pipeline for rendering to the render target
        let shader = Shader::new(
            ctx,
            geom_shader::VERTEX,
            geom_shader::FRAGMENT,
            geom_shader::META,
        )
        .expect("Building offscreen shader failed");
        let pipeline = Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("a_pos", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("a_color", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("a_inst_pos", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("a_inst_rot", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("a_inst_scale", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("a_inst_color", VertexFormat::Float4, 1),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        Self {
            pipeline,
            draw_calls: vec![],
            missing_bindings: false,
            camera_pan: (0.0, 0.0),
            camera_zoom: 1.0,
        }
    }

    /// Upload a lyon path.
    ///
    /// Returns a reference that can be used to add instances.
    pub fn upload_path<P>(&mut self, path: P, color: Color, opacity: f32) -> Mesh
    where
        P: IntoIterator<Item = PathEvent>,
    {
        // Tessalate the path, converting it to vertices & indices
        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        {
            tessellator
                .tessellate(
                    path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut geometry, VertexCtor::new(color, opacity)),
                )
                .unwrap();
        }
        let vertices = geometry.vertices.clone();
        let indices = geometry.indices;

        // Create an OpenGL draw call for the path
        let draw_call = DrawCall {
            vertices,
            indices,
            bindings: None,
            instances: vec![],
            refresh_instances: false,
        };
        self.draw_calls.push(draw_call);

        // Tell the next render loop to create bindings for this
        self.missing_bindings = true;

        // Return the draw call in a newtype struct so it can be used as a reference
        Mesh(self.draw_calls.len() - 1)
    }

    /// Upload lyon geometry.
    ///
    /// Returns a reference that can be used to add instances.
    pub fn upload_buffers(&mut self, geometry: &VertexBuffers<Vertex, u16>) -> Result<Mesh> {
        let vertices = geometry.vertices.clone();
        let indices = geometry.indices.clone();

        // Create an OpenGL draw call for the path
        let draw_call = DrawCall {
            vertices,
            indices,
            bindings: None,
            instances: vec![],
            refresh_instances: false,
        };
        self.draw_calls.push(draw_call);

        // Tell the next render loop to create bindings for this
        self.missing_bindings = true;

        // Return the draw call in a newtype struct so it can be used as a reference
        Ok(Mesh(self.draw_calls.len() - 1))
    }

    /// Render the graphics.
    pub fn render(&mut self, ctx: &mut Context) {
        let (width, height) = ctx.screen_size();

        // Create bindings & update the instance vertices if necessary
        if self.missing_bindings {
            self.draw_calls.iter_mut().for_each(|dc| {
                // Create bindings if missing
                if dc.bindings.is_none() {
                    dc.create_bindings(ctx);
                }
            });

            self.missing_bindings = false;
        }

        // Render the pass to the render target
        ctx.begin_default_pass(PassAction::clear_color(0.4, 0.7, 1.0, 1.0));

        // Render the separate draw calls
        for dc in self.draw_calls.iter_mut() {
            // Only render when we actually have instances
            if dc.instances.is_empty() {
                continue;
            }

            let bindings = dc.bindings.as_ref().unwrap();
            if dc.refresh_instances {
                // Upload the instance positions
                bindings.vertex_buffers[1].update(ctx, &dc.instances);

                dc.refresh_instances = false;
            }

            ctx.apply_pipeline(&self.pipeline);
            ctx.apply_scissor_rect(0, 0, width as i32, height as i32);
            ctx.apply_bindings(bindings);
            ctx.apply_uniforms(&geom_shader::Uniforms {
                zoom: (self.camera_zoom / width, self.camera_zoom / height),
                pan: (self.camera_pan.0, self.camera_pan.1),
            });
            ctx.draw(0, dc.indices.len() as i32, dc.instances.len() as i32);
        }

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    /// Set the camera panning position.
    pub fn set_camera_pos(&mut self, x: f32, y: f32) {
        self.camera_pan.0 = x;
        self.camera_pan.1 = y;
    }

    /// Set the camera zooming.
    pub fn set_camera_zoom(&mut self, zoom: f32) {
        self.camera_zoom = zoom;
    }

    /// Bind the GameLisp functions.
    pub fn bind_functions(runtime: &Runtime) {
        runtime.run(|| {
            glsp::bind_rfn("set_camera_pos", rfn!(Self::set_camera_pos))?;
            glsp::bind_rfn("set_camera_zoom", rfn!(Self::set_camera_zoom))?;

            Ok(())
        });
    }
}

/// A single uploaded mesh as a draw call.
#[derive(Debug)]
struct DrawCall {
    /// Render vertices, build by lyon path.
    vertices: Vec<Vertex>,
    /// Render indices, build by lyon path.
    indices: Vec<u16>,
    /// Render bindings, generated on render loop if empty.
    bindings: Option<Bindings>,
    /// List of instances to render.
    instances: Vec<Instance>,
    /// Whether the instance information should be reuploaded to the GPU.
    refresh_instances: bool,
}

impl DrawCall {
    /// Create bindings if they are missing.
    fn create_bindings(&mut self, ctx: &mut Context) {
        // The vertex buffer of the vector paths
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &self.vertices);
        // The index buffer of the vector paths
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &self.indices);

        // A dynamic buffer that will contain all positions for all instances
        let instance_positions = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            MAX_MESH_INSTANCES * mem::size_of::<Instance>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instance_positions],
            index_buffer,
            images: vec![],
        };
        self.bindings = Some(bindings);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

rdata! {
/// Instance of a mesh.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Instance {
    position: [f32; 3],
    rotation: f32,
    scale: f32,
    color: [f32; 3],
    alpha: f32,
}

meths {
    get "x": Instance::x,
    set "x": Instance::set_x,
    get "y": Instance::y,
    set "y": Instance::set_y,
    get "z": Instance::z,
    set "z": Instance::set_z,
    get "rotation": Instance::rotation,
    set "set_rotation": Instance::set_rotation,
    get "color_multiplier": Instance::color_multiplier,
    set "set_color_multiplier": Instance::set_color_multiplier,
}
}

impl Instance {
    /// Create a new instance with a position.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y, 0.0],
            rotation: 0.0,
            scale: 1.0,
            color: [1.0, 1.0, 1.0],
            alpha: 1.0,
        }
    }

    /// Set the X position.
    pub fn set_x(&mut self, new: f32) {
        self.position[0] = new;
    }

    /// Get the X position.
    pub fn x(&self) -> f32 {
        self.position[0]
    }

    /// Set the Y position.
    pub fn set_y(&mut self, new: f32) {
        self.position[1] = new;
    }

    /// Get the Y position.
    pub fn y(&self) -> f32 {
        self.position[1]
    }

    /// Set the Z position.
    pub fn set_z(&mut self, new: u8) {
        self.position[2] = (u8::MAX - new) as f32 / 255.0;
    }

    /// Get the Z position.
    pub fn z(&self) -> u8 {
        u8::MAX - (self.position[2] * 255.0) as u8
    }

    /// Set the scale.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Get the scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Set the rotation.
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Get the rotation.
    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Set the color multiplier.
    pub fn set_color_multiplier(&mut self, r: f32, g: f32, b: f32) {
        self.color = [r, g, b];
    }

    /// Get the color multiplier.
    pub fn color_multiplier(&self) -> (f32, f32, f32) {
        (self.color[0], self.color[1], self.color[2])
    }
}

/// Used by lyon to create vertices.
pub struct VertexCtor {
    color: [f32; 4],
}

impl VertexCtor {
    pub fn new(color: Color, alpha: f32) -> Self {
        Self {
            color: [
                color.red as f32 / 255.0,
                color.green as f32 / 255.0,
                color.blue as f32 / 255.0,
                alpha,
            ],
        }
    }
}

impl FillVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, position: Point, _: FillAttributes) -> Vertex {
        Vertex {
            pos: position.to_array(),
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, position: Point, _: StrokeAttributes) -> Vertex {
        Vertex {
            pos: position.to_array(),
            color: self.color,
        }
    }
}

mod geom_shader {
    use miniquad::graphics::*;

    pub const VERTEX: &str = r#"#version 100

uniform vec2 u_zoom;
uniform vec2 u_pan;

attribute vec2 a_pos;
attribute vec4 a_color;
attribute vec3 a_inst_pos;
attribute float a_inst_rot;
attribute float a_inst_scale;
attribute vec4 a_inst_color;

varying lowp vec4 color;

void main() {
    // Rotate vertices around the zero center
    float s = sin(a_inst_rot);
    float c = cos(a_inst_rot);
    mat2 rotation_mat = mat2(c, -s, s, c);
    vec2 rotated_pos = a_pos * rotation_mat;

    // Scale the rotated vertices
    vec2 scaled_pos = rotated_pos * a_inst_scale;

    // Offset scaled position with instance position
    // Offset with the camera multiplied by the Z position
    vec2 pos = scaled_pos + a_inst_pos.xy + u_pan * a_inst_pos.z;

    gl_Position = vec4(pos * vec2(1.0, -1.0) * u_zoom, a_inst_pos.z, 1.0);

    color = a_color * a_inst_color;
}
"#;

    pub const FRAGMENT: &str = r#"#version 100

varying lowp vec4 color;

void main() {
    gl_FragColor = color;
}
"#;

    pub const META: ShaderMeta = ShaderMeta {
        images: &[],
        uniforms: UniformBlockLayout {
            uniforms: &[
                UniformDesc::new("u_zoom", UniformType::Float2),
                UniformDesc::new("u_pan", UniformType::Float2),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub zoom: (f32, f32),
        pub pan: (f32, f32),
    }
}
