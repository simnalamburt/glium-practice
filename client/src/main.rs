extern crate common;
extern crate time;
extern crate xmath;
#[macro_use] extern crate glium;
extern crate rand;
extern crate obj;

mod draw_context;
mod traits;
mod error;
mod units;
mod resource;

use draw_context::DrawContext;
use std::default::Default;
use time::PreciseTime;
use units::{Nemo, Minion, MinionController};

#[cfg_attr(test, allow(dead_code))]
fn main() {
    // Make a render targets
    let (width, height) = (1024, 768);

    let display = (|| {
        for &depth in &[32u8, 24, 16] {
            use glium::DisplayBuild;

            let result = glium::glutin::WindowBuilder::new()
                .with_dimensions(width, height)
                .with_depth_buffer(depth)
                .with_title(common::PROJECT_NAME.to_string())
                .build_glium();

            match result {
                Ok(dp) => return dp,
                Err(_) => continue
            }
        }
        panic!("Failed to initialize glutin window");
    })();

    // TODO: Error 처리
    let draw_context = DrawContext::new(&display, width, height).unwrap();

    //
    // Basics
    //
    let (width, height) = (width as f32, height as f32);



    //
    // Game
    //
    let mut nemo = Nemo::new(&display).unwrap();
    let mut minions = vec![
        Minion::new(&display, (-17.0, 4.0)).unwrap(),
        Minion::new(&display, (-19.0, 2.0)).unwrap(),
        Minion::new(&display, (-20.0, 0.0)).unwrap(),
        Minion::new(&display, (-19.0,-2.0)).unwrap(),
        Minion::new(&display, (-17.0,-4.0)).unwrap(),
    ];
    let mut controller = MinionController::new(&display).unwrap();


    //
    // Parameters for UI
    //
    let vb_ui = glium::VertexBuffer::new(&display, &{
        #[derive(Clone, Copy)]
        struct Vertex { position: [f32; 2] }

        implement_vertex!(Vertex, position);

        vec![
            Vertex { position: [ -2.0, -2.0 ] },
            Vertex { position: [ -2.0,  3.0 ] },
            Vertex { position: [  3.0, -2.0 ] },
            Vertex { position: [  3.0,  3.0 ] },
        ]
    }).unwrap();
    let ib_ui = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
    let program_ui = glium::Program::from_source(&display,
        r#"
            #version 410
            uniform vec2 cursor;
            uniform mat4 matrix;
            in vec2 position;

            void main() {
                gl_Position = matrix * vec4(position + cursor, 0.0, 1.0);
            }
        "#, r#"
            #version 410
            out vec3 color;

            void main() {
                color = vec3(1.0, 1.0, 1.0);
            }
        "#, None).unwrap();
    let mut cursor = (300.0, 300.0);
    let matrix_ui = xmath::Matrix::orthographic_off_center(0.0, width, 0.0, height, 0.0, 1.0);


    let mut last = PreciseTime::now();

    // the main loop
    // each cycle will draw once
    'main: loop {
        use glium::Surface;
        use traits::Object;

        //
        // Poll and handle the events received by the window
        //
        for event in display.poll_events() {
            use glium::glutin::{Event, ElementState, MouseButton};
            use glium::glutin::VirtualKeyCode as vkey;

            match event {
                Event::MouseMoved((x, y)) => cursor = (x as f32, height - y as f32),
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
                    use traits::Move;

                    // 마우스 좌표계 ~ 게임 좌표계 변환
                    let dest = ((cursor.0 - width/2.0)/10.0, (cursor.1 - height/2.0)/10.0);
                    nemo.go(dest)
                }
                Event::MouseInput(ElementState::Pressed, MouseButton::Right) => {
                    let texture = &draw_context.texture_for_object_picking;
                    let mut object_picking_buffer = texture.as_surface();
                    object_picking_buffer.clear_color(1.0, 1.0, 1.0, 1.0);
                    // TODO: 예외처리
                    nemo.fill(&mut object_picking_buffer, &draw_context).unwrap();
                    for minion in &minions {
                        minion.fill(&mut object_picking_buffer, &draw_context).unwrap();
                    }
                    controller.fill(&mut object_picking_buffer, &draw_context).unwrap();
                    let buffer = texture.read_to_pixel_buffer();
                    let pixel_index = (width * cursor.1 + cursor.0) as usize;
                    let pixel_color = buffer.slice(pixel_index..(pixel_index + 1)).unwrap().read().unwrap()[0];

                    println!("{:?} {:?}", cursor, color_to_id(&pixel_color));
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(vkey::Q)) => nemo.q(),
                Event::Closed => break 'main,
                _ => ()
            }
        }


        //
        // Update
        //
        // TODO: Limit framerate
        let now = PreciseTime::now();
        let delta = last.to(now).num_nanoseconds().unwrap() as f32 / 1.0E+9;
        last = now;

        nemo.update(delta);
        for m in &mut minions {
            m.update(delta);
        }
        controller.update(delta);


        //
        // Render
        //
        let uniforms_ui = uniform! {
            cursor: cursor,
            matrix: matrix_ui.clone()
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        nemo.draw(&mut target, &draw_context).unwrap();
        for minion in &minions {
            minion.draw(&mut target, &draw_context).unwrap();
        }
        controller.draw(&mut target, &draw_context).unwrap();

        target.draw(&vb_ui, &ib_ui, &program_ui, &uniforms_ui, &Default::default()).unwrap();
        let _ = target.finish();
    }
}

fn color_to_id(color: &(u8, u8, u8, u8)) -> Option<u32> {
    match *color {
        (255, 255, 255, 255) => None,
        (red, green, blue, alpha) => {
            let red = (red as u32) << 24;
            let green = (green as u32) << 16;
            let blue = (blue as u32) << 8;
            let alpha = alpha as u32;
            Some(red | green | blue | alpha)
        }
    }
}
