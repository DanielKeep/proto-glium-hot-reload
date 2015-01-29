#![allow(unstable)]
#![feature(plugin)]

#[macro_use] extern crate glium;
#[plugin] extern crate glium_macros;
extern crate glutin;
extern crate typemap;
extern crate sandbox_abi;

use glium::Surface;
use typemap::TypeMap;
use sandbox_abi::{DisplayKey, Frozen, Reload, Renderer};

macro_rules! lazy_member {
    (fn $mname:ident (&mut $this:ident) :: $fname:ident: $fty:ty = $init:expr) => {
        fn $mname(&mut $this) -> &$fty {
            match $this.$fname {
                Some(ref value) => value,
                None => {
                    let value = $init;
                    $this.$fname = Some(value);
                    $this.$fname.as_ref().unwrap()
                }
            }
        }
    };
}

struct GliumRenderer {
    display: glium::Display,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer,
    program: glium::Program,
}

#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl GliumRenderer {
    pub fn new(deps: &TypeMap, frozen: Option<TypeMap>) -> GliumRenderer {
        println!("GliumRenderer::new(#deps: {:?}, #frozen: {:?})", deps.len(), frozen.as_ref().map(|tm| tm.len()));

        let _ = frozen;

        println!("DisplayKey type_id: {:?}", ::std::any::TypeId::of::<DisplayKey>());
        let display = deps.find::<DisplayKey>().expect("expected glium::Display in deps").clone();

        let (vertex_buffer, index_buffer, program) = GliumRenderer::load(&display);

        GliumRenderer {
            display: display,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
        }
    }

    fn load(display: &glium::Display) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer, glium::Program) {
        let vertex_buffer = glium::VertexBuffer::new(
            display,
            vec![
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        );

        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index_buffer::TrianglesList(vec![0u16, 1, 2])
        );

        let program = glium::Program::from_source(
            display,
            // vertex shader
            r###"
                #version 110

                uniform mat4 matrix;

                attribute vec2 position;
                attribute vec3 color;

                varying vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            "###,

            // fragment shader
            r###"
                #version 110
                varying vec3 vColor;

                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            "###,

            // geometry shader
            None
        ).unwrap();

        (vertex_buffer, index_buffer, program)
    }
}

impl Reload for GliumRenderer {
    fn freeze(self: Box<Self>) -> Frozen {
        println!("GliumRenderer::freeze()");
        TypeMap::new()
    }
}

impl Renderer for GliumRenderer {
    // fn poll_events(&mut self) -> Box<Iterator<Item=glutin::Event>> {
    //     Box::new(self.display.poll_events())
    // }

    fn render(&mut self) {
        let uniforms = uniform! {
            matrix: [
                [0.0, 1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &::std::default::Default::default()).ok();
        target.finish();
    }
}

#[no_mangle]
pub fn module_factory(deps: &TypeMap, frozen: Option<TypeMap>) -> Box<Renderer + 'static> {
    Box::new(GliumRenderer::new(deps, frozen)) as Box<Renderer>
}
