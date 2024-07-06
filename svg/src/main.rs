use std::{env, thread};

use svg::node::element::path::{Command, Position};
// use svg::node::element::{Path, Rectangle};
use svg::Document;

use crossbeam::unbounded;

// use rayon::prelude::*;

// mod channels;

const WIDTH: isize = 400;
const HEIGHT: isize = WIDTH;

const HOME_X: isize = WIDTH / 2;
const HOME_Y: isize = HEIGHT / 2;

// const STROKE_WIDTH: usize = 5;

#[derive(Debug, Clone, Copy)]
enum Operation {
    Forward(isize),
    TurnLeft,
    TurnRight,
    Home,
    Noop(u8),
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    North,
    South,
    West,
    East,
}

#[derive(Debug)]
struct Artist {
    x: isize,
    y: isize,
    heading: Orientation,
}

impl Artist {
    fn new() -> Self {
        Artist {
            x: HOME_X,
            y: HOME_Y,
            heading: Orientation::North,
        }
    }

    fn home(&mut self) {
        self.x = HOME_X;
        self.y = HOME_Y;
    }

    fn forward(&mut self, distance: isize) {
        match self.heading {
            Orientation::North => self.y += distance,
            Orientation::South => self.y -= distance,
            Orientation::West => self.x += distance,
            Orientation::East => self.x -= distance,
        }
    }

    fn turn_left(&mut self) {
        self.heading = match self.heading {
            Orientation::North => Orientation::West,
            Orientation::South => Orientation::East,
            Orientation::West => Orientation::South,
            Orientation::East => Orientation::North,
        }
    }

    fn turn_right(&mut self) {
        self.heading = match self.heading {
            Orientation::North => Orientation::East,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North,
            Orientation::East => Orientation::South,
        }
    }

    fn wrap(&mut self) {
        if self.x < 0 {
            self.x = HOME_X;
            self.heading = Orientation::West;
        } else if self.x > WIDTH {
            self.x = HOME_X;
            self.heading = Orientation::East;
        }

        if self.y < 0 {
            self.y = HOME_Y;
            self.heading = Orientation::North;
        } else if self.y > HEIGHT {
            self.y = HOME_Y;
            self.heading = Orientation::South;
        }
    }
}

enum Work {
    Task((usize, u8)),
    Finished,
}

fn parse_byte(byte: u8) -> Operation {
    match byte {
        b'0' => Operation::Home,
        b'1'..=b'9' => {
            let distance = (byte - 0x30) as isize; // 1
            Operation::Forward(distance * (HEIGHT / 10))
        }
        b'a' | b'b' | b'c' => Operation::TurnLeft,  // 2
        b'd' | b'e' | b'f' => Operation::TurnRight, // 2
        _ => Operation::Noop(byte),                 // 3
    }
}

fn parse(input: &str) -> Vec<Operation> {
    // let mut steps = Vec::<Operation>::new();
    // for byte in input.bytes() {
    //     let step = match byte {
    //         b'0' => Operation::Home,
    //         b'1'..=b'9' => {
    //             let distance = (byte - 0x30) as isize; // 1
    //             Operation::Forward(distance * (HEIGHT / 10))
    //         }
    //         b'a' | b'b' | b'c' => Operation::TurnLeft, // 2
    //         b'd' | b'e' | b'f' => Operation::TurnRight, // 2
    //         _ => Operation::Noop(byte),                // 3
    //     };
    //     steps.push(step);
    // }
    // steps

    // input
    //     .bytes()
    //     .map(|byte| {
    //         match byte {
    //             b'0' => Operation::Home,
    //             b'1'..=b'9' => {
    //                 let distance = (byte - 0x30) as isize; // 1
    //                 Operation::Forward(distance * (HEIGHT / 10))
    //             }
    //             b'a' | b'b' | b'c' => Operation::TurnLeft, // 2
    //             b'd' | b'e' | b'f' => Operation::TurnRight, // 2
    //             _ => Operation::Noop(byte),                // 3
    //         }
    //     })
    //     .collect()

    // input
    //     .as_bytes()
    //     .par_iter()
    //     .map(|byte| match byte {
    //         b'0' => Operation::Home,
    //         b'1'..=b'9' => {
    //             let distance = (byte - 0x30) as isize; // 1
    //             Operation::Forward(distance * (HEIGHT / 10))
    //         }
    //         b'a' | b'b' | b'c' => Operation::TurnLeft, // 2
    //         b'd' | b'e' | b'f' => Operation::TurnRight, // 2
    //         _ => Operation::Noop(*byte),               // 3
    //     })
    //     .collect()

    let n_threads = 2;
    let (todo_tx, todo_rx) = unbounded();
    let (results_tx, results_rx) = unbounded();
    let mut n_bytes = 0;
    for (i, byte) in input.bytes().enumerate() {
        todo_tx.send(Work::Task((i, byte))).unwrap();
        n_bytes += 1;
    }

    for _ in 0..n_threads {
        todo_tx.send(Work::Finished).unwrap();
    }

    for _ in 0..n_threads {
        let todo = todo_rx.clone();
        let results = results_tx.clone();

        thread::spawn(move || loop {
            let task = todo.recv();
            let result = match task {
                Err(_) => break,
                Ok(Work::Finished) => break,
                Ok(Work::Task((i, byte))) => (i, parse_byte(byte)),
            };
            results.send(result).unwrap();
        });
    }

    let mut ops = vec![Operation::Noop(0); n_bytes];

    for _ in 0..n_bytes {
        let (i, op) = results_rx.recv().unwrap();
        ops[i] = op;
    }

    ops
}

fn convert(operations: &Vec<Operation>) -> Vec<Command> {
    let mut turtle = Artist::new();
    let mut path_data = Vec::<Command>::with_capacity(1 + operations.len());
    let start_at_home = Command::Move(Position::Absolute, (HOME_X, HOME_Y).into()); // 1
    path_data.push(start_at_home);

    for op in operations {
        match *op {
            Operation::Forward(distance) => turtle.forward(distance), // 2
            Operation::TurnLeft => turtle.turn_left(),                // 2
            Operation::TurnRight => turtle.turn_right(),              // 2
            Operation::Home => turtle.home(),                         // 2
            Operation::Noop(byte) => {
                eprintln!("warning: illegal byte encountered: {:?}", byte)
            }
        };

        let line = Command::Line(Position::Absolute, (turtle.x, turtle.y).into()); // 3
        path_data.push(line);

        turtle.wrap(); // 4
    }

    path_data
}

fn generate_svg(_path_data: Vec<Command>) -> Document {
    // let background = Rectangle::new();

    // let border = background.clone();

    // let sketch = Path::new();

    let document = Document::new();

    document
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let input = args.get(1).unwrap();
    let default_filename = format!("{}.svg", input);
    let save_to = args.get(2).unwrap_or(&default_filename);

    let operations = parse(input);
    let path_data = convert(&operations);
    let document = generate_svg(path_data);
    svg::save(save_to, &document).unwrap();
}
