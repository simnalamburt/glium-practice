use glium::{VertexBuffer, Program, Frame};
use glium::index::*;
use glium::backend::Facade;
use xmath::Matrix;
use traits::*;

pub struct Minion {
    vb: VertexBuffer<Vertex>,
    ib: NoIndices,
    program: Program,
    pos: (f32, f32),
    angle: f32,
    state: State,
}

#[derive(Clone, Copy)]
struct Vertex { position: [f32; 2] }

implement_vertex!(Vertex, position);

enum State {
    /// Nemo is stopped
    Stopped,
    /// Nemo is moving
    Moving { dest: (f32, f32) },
}

impl Minion {
    pub fn new<F: Facade>(facade: &F, pos: (f32, f32)) -> Self {
        Minion {
            vb: VertexBuffer::new(facade, {
                vec![
                    Vertex { position: [  2.0,  0.00 ] },
                    Vertex { position: [ -2.0,  0.75 ] },
                    Vertex { position: [ -2.0, -0.75 ] },
                ]
            }),
            ib: NoIndices(PrimitiveType::TriangleStrip),
            program: Program::from_source(facade, r#"
                #version 410
                uniform mat4 matrix;
                in vec2 position;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                }
            "#, r#"
                #version 410
                out vec3 color;

                void main() {
                    color = vec3(1.0, 0.5, 0.5);
                }
            "#, None).unwrap(),
            pos: pos,
            angle: 0.0,
            state: State::Stopped,
        }
    }
}

impl Object for Minion {
    fn update(&mut self, elapsed: f32) {
        // Does nothing
        let mut next = None;

        match self.state {
            State::Stopped => {}
            State::Moving { dest } => {
                let dx = dest.0 - self.pos.0;
                let dy = dest.1 - self.pos.1;

                let left_dist = (dx*dx + dy*dy).sqrt();

                let speed = 50.0;
                let diff = speed*elapsed;

                if left_dist <= diff {
                    // 도착
                    self.pos = dest;
                    next = Some(State::Stopped);
                } else {
                    self.pos.0 += diff*self.angle.cos();
                    self.pos.1 += diff*self.angle.sin();
                }
            }
        };

        next.map(|next| {
            self.state = next;
        });
    }

    fn draw(&self, mut target: Frame, camera: Matrix) -> Frame {
        use glium::Surface;

        // TODO: Cache
        let local = Matrix::rotation_z(self.angle);
        let world = Matrix::translation(self.pos.0, self.pos.1, 0.0);

        let uniforms = uniform! {
            matrix: local * world * camera,
        };

        target.draw(&self.vb, &self.ib, &self.program, &uniforms, &Default::default()).unwrap();
        target
    }
}

impl Unit for Minion {
    fn go(&mut self, dest: (f32, f32)) {
        if self.pos == dest { return; }

        let dx = dest.0 - self.pos.0;
        let dy = dest.1 - self.pos.1;
        self.angle = dy.atan2(dx);
        self.state = State::Moving { dest: dest };
    }
}


/// 미니언을 조종하는 객체
pub struct MinionController {
    minions: Vec<Minion>
}

impl MinionController {
    pub fn new<F: Facade>(facade: &F) -> Self {
        use rand;
        use rand::distributions::{IndependentSample, Range};

        let mut minions = vec![
            Minion::new(facade, (17.0, 4.0)),
            Minion::new(facade, (19.0, 2.0)),
            Minion::new(facade, (20.0, 0.0)),
            Minion::new(facade, (19.0,-2.0)),
            Minion::new(facade, (17.0,-4.0)),
        ];

        let range = Range::new(-10.0, 10.0);
        let mut rng = rand::thread_rng();

        for minion in &mut minions {
            let x = range.ind_sample(&mut rng);
            let y = range.ind_sample(&mut rng);
            minion.go((x, y));
        }

        MinionController { minions: minions }
    }
}

impl Object for MinionController {
    fn update(&mut self, elapsed: f32) {
        for minion in &mut self.minions {
            minion.update(elapsed);
        }
    }

    fn draw(&self, target: Frame, camera: Matrix) -> Frame {
        self.minions.iter().fold(target, |target, ref minion| {
            minion.draw(target, camera.clone())
        })
    }
}
