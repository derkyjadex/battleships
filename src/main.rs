#[macro_use]
extern crate glium;
extern crate alice;
extern crate aldata;

use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseButton};
use alice::model::rendering::{ModelRenderer, prepare_model};
use alice::model::{Model, Path, Point};
use std::io::{Read, Write};
//use aldata::{Vec2, Vec3};

use std::net::{TcpListener, TcpStream};

fn main() {
    let mode = std::env::args().nth(1).unwrap();
    if mode == "server" {
        server();
    } else if mode == "client" {
        client();
    } else {
        println!("choose client or server");
    }
}

fn server() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for stream in listener.incoming() {
        run(&mut stream.unwrap(), true);
    }
}

fn client() {
    let mut stream = TcpStream::connect("127.0.0.1:1234").unwrap();
    run(&mut stream, false);
}

fn run(stream: &mut TcpStream, starting: bool) {
    let mine = MyBoard {
        squares: [
            [false, false, false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false, false, false],
            [false, false, false,  true,  true,  true, false, false,  true, false],
            [false, false, false, false, false, false, false, false,  true, false],
            [false, false, false, false, false, false, false, false,  true, false],
            [false, false, false, false, false, false,  true, false, false, false],
            [false, false, false, false, false, false,  true, false, false, false],
            [false,  true,  true, false, false, false,  true, false, false, false],
            [false, false, false, false, false, false,  true, false, false, false],
            [false, false, false, false, false, false, false, false, false, false],
            ]
    };

    let mut theirs = TheirBoard {
        squares: [[FireResult::None; SIZE]; SIZE]
    };

    let mut bombs = Bombs::new();

    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Battleships"))
        .with_dimensions(1050, 500)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    let mut renderer = ModelRenderer::new(&display);
    renderer.set_size(1050.0, 500.0);

    let mut mouse_pos = (0, 0);
    let mut our_turn = starting;

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let mine_model = prepare_model(&display, &mine.render());
        renderer.draw(&mut target, 0.0, 0.0, 1.0, &mine_model);

        let theirs_model = prepare_model(&display, &theirs.render());
        renderer.draw(&mut target, 550.0, 0.0, 1.0, &theirs_model);

        let bombs_model = prepare_model(&display, &bombs.render());
        renderer.draw(&mut target, 0.0, 0.0, 1.0, &bombs_model);

        target.finish().unwrap();

        if !our_turn {
            let mut buffer = [0; 4];
            assert_eq!(4, stream.read(&mut buffer).unwrap());
            let i = (buffer[0] - b'0') as usize;
            let j = (buffer[2] - b'0') as usize;
            if mine.squares[i][j] {
                let buffer = "h\n".bytes().collect::<Vec<_>>();
                stream.write_all(&buffer).unwrap();
            } else {
                let buffer = "m\n".bytes().collect::<Vec<_>>();
                stream.write_all(&buffer).unwrap();
            }
            bombs.push(i, j);

            our_turn = true;
        }

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                Event::MouseMoved(pos) => {
                    mouse_pos = pos;
                },
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) if our_turn => {
                    let f = window.hidpi_factor();
                    let (x, y) = mouse_pos;
                    let x = x as f32 / f;
                    let y = y as f32 / f;
                    let y = 500.0 - y;
                    let x = x - 550.0;

                    let i = y as usize / 50;
                    let j = x as usize / 50;
                    if i > 9 || j > 9 {
                        break
                    }
                    let buffer = format!("{},{}\n", i, j).bytes().collect::<Vec<_>>();
                    stream.write_all(&buffer).unwrap();

                    let mut buffer = [0; 2];
                    stream.read(&mut buffer).unwrap();
                    let result = match buffer[0] {
                        b'm' => FireResult::Miss,
                        b'h' => FireResult::Hit,
                        _ => FireResult::None,
                    };
                    theirs.squares[i][j] = result;
                    our_turn = false;
                }
                _ => ()
            }
        }
    }
}


const SIZE: usize = 10;

struct MyBoard {
    squares: [[bool; SIZE]; SIZE]
}

impl MyBoard {
    pub fn render(&self) -> Model {
        let mut paths = Vec::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                let colour = if self.squares[i][j] {
                    (0.1, 0.1, 0.9)
                } else {
                    (0.1, 0.1, 0.1)
                };
                let points = vec![
                    Point {
                        location: (j as f64 * 50.0, i as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: ((j + 1) as f64 * 50.0, i as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: ((j + 1) as f64 * 50.0, (i + 1) as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: (j as f64 * 50.0, (i + 1) as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    ];
                paths.push(Path {
                    colour: colour,
                    points: points
                });
            }
        }

        Model {
            paths: paths
        }
    }
}

#[derive(Copy, Clone)]
enum FireResult {
    None,
    Miss,
    Hit
}

struct TheirBoard {
    squares: [[FireResult; SIZE]; SIZE]
}

impl TheirBoard {
    pub fn render(&self) -> Model {
        let mut paths = Vec::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                let colour = match self.squares[i][j] {
                    FireResult::None => (0.1, 0.1, 0.1),
                    FireResult::Miss => (0.9, 0.1, 0.1),
                    FireResult::Hit => (0.9, 0.9, 0.1),
                };
                let points = vec![
                    Point {
                        location: (j as f64 * 50.0, i as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: ((j + 1) as f64 * 50.0, i as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: ((j + 1) as f64 * 50.0, (i + 1) as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    Point {
                        location: (j as f64 * 50.0, (i + 1) as f64 * 50.0),
                        curve_bias: 0.0
                    },
                    ];
                paths.push(Path {
                    colour: colour,
                    points: points
                });
            }
        }

        Model {
            paths: paths
        }
    }
}

struct Bombs {
    bombs: Vec<(usize, usize)>
}

impl Bombs {
    pub fn new() -> Bombs {
        Bombs { bombs: Vec::new() }
    }

    pub fn render(&self) -> Model {
        let mut paths = Vec::new();
        for (i, j) in self.bombs.clone() {
            let colour = (0.9, 0.7, 0.0);
            let points = vec![
                Point {
                    location: (j as f64 * 50.0, i as f64 * 50.0),
                    curve_bias: 0.5
                },
                Point {
                    location: ((j + 1) as f64 * 50.0, i as f64 * 50.0),
                    curve_bias: 0.5
                },
                Point {
                    location: ((j + 1) as f64 * 50.0, (i + 1) as f64 * 50.0),
                    curve_bias: 0.5
                },
                Point {
                    location: (j as f64 * 50.0, (i + 1) as f64 * 50.0),
                    curve_bias: 0.5
                },
            ];

            paths.push(Path {
                colour: colour,
                points: points
            })
        }

        Model {
            paths: paths
        }
    }

    pub fn push(&mut self, i: usize, j: usize) {
        self.bombs.push((i, j));
    }
}
