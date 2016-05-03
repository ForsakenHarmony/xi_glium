
use std::path::Path;
use std::fs::File;

use glium;
use glium_text;
use glium::Surface;
use glium::index::PrimitiveType;

use text::Line;

pub struct Target<'a> {
    target: glium::Frame,
    renderer: &'a Renderer,
}

impl<'a> Target<'a> {
    pub fn finish(self) {
        self.target.finish().unwrap();
    }
}

pub struct Renderer {
    display: glium::backend::glutin_backend::GlutinFacade,
    program: glium::Program,
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,
}

impl Renderer {
    pub fn new(display: glium::backend::glutin_backend::GlutinFacade) -> Renderer {
        let font_size = 15;

        let text_system = glium_text::TextSystem::new(&display);
        let font_texture = glium_text::FontTexture::new(&display, File::open(&Path::new("Hack-Regular.ttf")).unwrap(), font_size).unwrap();

        let program = program!(&display,
            140 => {
                vertex: "
                    #version 140
                    in vec2 position;
                    in vec4 color;
                    out vec4 v_color;
                    uniform vec2 win_size;
                    uniform vec2 offset;
                    void main() {
                        v_color = color;
                        gl_Position = vec4((position + offset) / win_size * 2. - 1., 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 140
                    in vec4 v_color;
                    out vec4 color;
                    void main() {
                        color = v_color;
                    }
                "
            },
            110 => {
                vertex: "
                    #version 110

                    attribute vec2 position;
                    attribute vec4 color;
                    varying vec4 v_color;

                    uniform vec2 win_size;
                    uniform vec2 offset;

                    void main() {
                        v_color = color;
                        gl_Position = vec4((position + offset) / win_size * 2. - 1., 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 110

                    varying vec4 v_color;

                    void main() {
                        gl_FragColor = v_color;
                    }
                "
        }).unwrap();


        let renderer = Renderer {
            display: display,
            program: program,
            text_system: text_system,
            font_texture: font_texture,
        };

        renderer
    }

    pub fn draw(&self) -> Target {
        let mut target = self.display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);
        Target { target: target, renderer: &self }
    }
}

pub struct LineRenderer<'a> {
    text_display: glium_text::TextDisplay<&'a glium_text::FontTexture>,
    pub char_pos_x: Vec<f32>, // in screen coordinates
}

impl<'a> LineRenderer<'a> {
    pub fn new(renderer: &'a Renderer, text: &str) -> LineRenderer<'a> {
        let text_display = glium_text::TextDisplay::new(&renderer.text_system, &renderer.font_texture, text);
        let em_pixels = renderer.font_texture.em_pixels() as f32;
        let char_pos_x = text_display.get_char_pos_x().into_iter().map(|&x| x * em_pixels).collect();

        LineRenderer {
            text_display: text_display,
            char_pos_x: char_pos_x,
        }
    }

    pub fn draw(&self, target: &mut Target, px: f32, py: f32) {
        let size = target.renderer.font_texture.em_pixels();
        let (w, h) = target.target.get_dimensions();
        let text_tf = |px: f32, py: f32| -> [[f32; 4]; 4] {
            let (x, y) = (px / w as f32 * 2. - 1.,
                         (py - size as f32 / 2.) / h as f32 * 2. - 1.);

            let scale = 2. * size as f32;

            [[scale / w as f32, 0.0, 0.0, 0.0],
             [0.0, scale / h as f32, 0.0, 0.0],
             [0.0,              0.0, 1.0, 0.0],
             [  x,                y, 0.0, 1.0]]
        };
        glium_text::draw(&self.text_display, &target.renderer.text_system, &mut target.target, text_tf(px, py), (0., 0., 0., 1.));
    }
}

// This struct and impl is in fact isolated from the renderer backend, it can be separated into a file
pub struct TextRenderer {
    cursor: Primitive,
    line_bg: Primitive,
    left_margin: f32,
}

impl TextRenderer {
    pub fn new(renderer: &Renderer, left_margin: f32) -> TextRenderer {
        let cursor = Primitive::new_line(&renderer, (0.,-10.), (0.,10.), [0.,0.,0.,1.]);
        let line_bg = Primitive::new_rect(&renderer, (0., -10.), (2000., 10.), [1.,1.,0.7,1.]);

        TextRenderer { cursor: cursor, line_bg: line_bg, left_margin: left_margin }
    }

    pub fn draw_line(&self, target: &mut Target, line: &Line, (px, py): (f32, f32))
            -> Result<(), glium::DrawError> {

        if let Some(mut pos) = line.cursor {
            let ch_pos_x = &line.renderer.char_pos_x;
            assert!(ch_pos_x.len() > pos as usize);
            let offset = ch_pos_x[pos as usize];

            self.line_bg.draw(target, (px, py)).unwrap();
            self.cursor.draw(target, (offset + px, py)).unwrap();
        }

        line.renderer.draw(target, px, py);

        Ok(())
    }

    pub fn draw(&self, target: &mut Target, lines: &[(f32,&Line)]) {
        for &(y, line) in lines {
            self.draw_line(target, &line, (self.left_margin, y));
        }
        // let (w,h) = target.target.get_dimensions();
        // self.renderer.draw_scrollbar(target, w - 20., h, 0.);
    }

    // pub fn draw_scrollbar(&self, target: &mut Target, x: f32, y1: f32, y2: f32, top: f64, height: f64, total: f64)
    //         -> Result<(), glium::DrawError> {
    //     const WIDTH: f32 = 15.;
    //     let mesh = Primitive::new_rect(&target.renderer, (x-WIDTH/2., 100.), (x+WIDTH/2., 1000.), [0.4,0.4,0.4,1.]);
    //     mesh.draw(target, (0., 0.));
    //     unimplemented!()
    // }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

pub struct Primitive {
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer:  glium::index::NoIndices,
    fill: bool,
}

impl Primitive {
    // pub fn new(renderer: &Renderer, verts: &[Vertex], primitive_type: glium::index::PrimitiveType, fill: bool) -> Self {
    //     Primitive {
    //         vertex_buffer: glium::VertexBuffer::new(&renderer.display, verts).unwrap(),
    //         index_buffer: glium::index::NoIndices(primitive_type),
    //         fill: fill,
    //     }
    // }

    pub fn new_rect(renderer: &Renderer, p1: (f32,f32), p2: (f32,f32), color: [f32; 4]) -> Self {
        let verts = vec![
            Vertex { position: [p1.0, p1.1], color: color },
            Vertex { position: [p2.0, p1.1], color: color },
            Vertex { position: [p1.0, p2.1], color: color },
            Vertex { position: [p2.0, p2.1], color: color },
        ];
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(&renderer.display, &verts).unwrap(),
            index_buffer:  glium::index::NoIndices(PrimitiveType::TriangleStrip),
            fill: true,
        }
    }

    pub fn new_line(renderer: &Renderer, p1: (f32,f32), p2: (f32,f32), color: [f32; 4]) -> Self {
        let verts = vec![
            Vertex { position: [p1.0, p1.1], color: color },
            Vertex { position: [p2.0, p2.1], color: color },
        ];
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(&renderer.display, &verts).unwrap(),
            index_buffer:  glium::index::NoIndices(PrimitiveType::LinesList),
            fill: false,
        }
    }

    pub fn draw(&self, target: &mut Target, offset: (f32, f32)) -> Result<(), glium::DrawError> {
        let (w, h) = target.target.get_dimensions();
        let params = glium::DrawParameters {
            polygon_mode: if self.fill { glium::draw_parameters::PolygonMode::Fill } else { glium::draw_parameters::PolygonMode::Line },
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        target.target.draw(&self.vertex_buffer, &self.index_buffer, &target.renderer.program, &uniform!{ win_size: (w as f32, h as f32), offset: offset }, &params)
    }
}
