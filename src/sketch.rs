use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

use nannou::prelude::*;
use nannou::wgpu::{Backends, DeviceDescriptor, Limits};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

pub enum ModelState {
    Idle,
    CalculatingShortestPath,
}

pub struct Model {
    graph: HashMap<u16, Vec<u16>>,
    positions: HashMap<u16, Point2>,
    left_clicked: u16,
    right_clicked: u16,
    predecessor: HashMap<u16, u16>,
    shortest_path: Vec<u16>,
    visited: HashSet<u16>,
    queue: VecDeque<u16>,
    state: ModelState,
}

impl Model {
    pub fn new_random(n: u16) -> Self {
        let mut positions = HashMap::new();
        for i in 0..n {
            let x = random_range(-(WIDTH as f32) / 2.0, WIDTH as f32 / 2.0);
            let y = random_range(-(HEIGHT as f32) / 2.0, HEIGHT as f32 / 2.0);
            positions.insert(i, Point2::new(x, y));
        }
        let mut graph = HashMap::new();
        for i in 0..n {
            let mut close_neighbors = Vec::new();
            for j in (0..n).filter(|&j| j != i ) {
                let distance = positions[&i].distance(positions[&j]);
                if distance < WIDTH as f32 / 10.0 {
                    close_neighbors.push(j);
                }
                
            }
            graph.insert(i, close_neighbors);
        }
        
        Model {
            graph,
            positions,
            left_clicked: 0,
            right_clicked: 1,
            predecessor: HashMap::new(),
            shortest_path: Vec::new(),
            visited: HashSet::new(),
            queue: VecDeque::from([0]),
            state: ModelState::CalculatingShortestPath,
        }
    }

    pub fn shortest_path_step(&mut self) {
        while let Some(node) = self.queue.pop_front() {
            if node == self.right_clicked {
                self.queue.clear();
                let mut path = vec![node];
                let mut current = node;
                while let Some(&predecessor) = self.predecessor.get(&current) {
                    path.push(current);
                    if predecessor == self.left_clicked {
                        path.push(predecessor);
                        path.reverse();
                        self.shortest_path = path;
                        self.state = ModelState::Idle;
                        return;
                    }
                    current = predecessor;
                }
            } 
            if self.visited.contains(&node) {
                continue;
            }
            self.visited.insert(node);
            for neighbor in &self.graph[&node] {
                if !self.visited.contains(neighbor) {
                    self.queue.push_back(*neighbor);
                    self.predecessor.insert(*neighbor, node);
                }
            }
            self.state = ModelState::CalculatingShortestPath;
            return;
        }
        self.state = ModelState::Idle;
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if app.mouse.buttons.left().is_down() {
        if let Some(closest) = model.positions.iter().min_by_key(|(_, pos)| pos.distance(app.mouse.position()).round() as u32) {
            model.left_clicked = *closest.0;
            model.visited.clear();
            model.shortest_path.clear();
            model.queue.clear();
            model.queue.push_back(model.left_clicked);
            model.predecessor.clear();
            model.state = ModelState::CalculatingShortestPath;
        }
    }
    if app.mouse.buttons.right().is_down() {
        if let Some(closest) = model.positions.iter().min_by_key(|(_, pos)| pos.distance(app.mouse.position()).round() as u32) {
            model.right_clicked = *closest.0;
            model.visited.clear();
            model.shortest_path.clear();
            model.queue.clear();
            model.queue.push_back(model.left_clicked);
            model.predecessor.clear();
            model.state = ModelState::CalculatingShortestPath;
        }
    }
    if let ModelState::CalculatingShortestPath = model.state {
        model.shortest_path_step();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(DARKGRAY);

    draw_model(&draw, model);
    draw_mouse_lines(app, &draw, model);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn draw_mouse_lines(app: &App, draw: &Draw, model: &Model) {
    let mouse_pos = app.mouse.position();
    for pos in model.positions.values().into_iter().filter(|next_pos| next_pos.distance(mouse_pos) < 200.0) {
        let color = LinSrgba::new(0.0, 0.0, 0.0, 1.0 - (200.0 / pos.distance(mouse_pos)));
        draw.line()
            .start(pt2(pos.x, pos.y))
            .end(pt2(mouse_pos.x, mouse_pos.y))
            .color(color);
    }
}

fn draw_model(draw: &Draw, model: &Model) {
    for (node, neighbors) in &model.graph {
        let pos = model.positions[node];
        let (color, z_index) = match node {
            _ if node == &model.left_clicked => (RED, 4.0),
            _ if node == &model.right_clicked => (BLUE, 4.0),
            _ => (WHITE, 3.0),
        };
        draw.ellipse()
            .x_y(pos.x, pos.y).w_h(10.0, 10.0)
            .color(color)
            .z(z_index);
        
        for j in neighbors {
            let (color, line_width) = if model.shortest_path.windows(2).find(|x| x[0] == *node && x[1] == *j || x[1] == *node && x[0] == *j).is_some() {
                (TEAL, 2.0)
            } else if model.visited.contains(node) {
                (RED, 1.0)
            } else {
                (WHITE, 1.0)
            };
            let neighbor_pos = model.positions[j];
            draw.line()
                .start(pt2(pos.x, pos.y))
                .end(pt2(neighbor_pos.x, neighbor_pos.y))
                .color(color)
                .stroke_weight(line_width)
                .z(line_width);
        }
    }
}

pub async fn run_app() {
    let model = Model::new_random(250);
    thread_local!(static MODEL: RefCell<Option<Model>> = Default::default());    
    MODEL.with(|m| m.borrow_mut().replace(model));

    app::Builder::new_async(|app| {
        Box::new(async move {
            create_window(app).await;
            MODEL.with(|m| m.borrow_mut().take().unwrap())
        })
    })
        .backends(Backends::PRIMARY | Backends::GL)
        .update(update)
        .run_async()
        .await;
}

async fn create_window(app: &App) {
    let device_desc = DeviceDescriptor {
        limits: Limits {
            max_texture_dimension_2d: 8192,
            ..Limits::downlevel_webgl2_defaults()
        },
        ..Default::default()
    };

    app.new_window()
        .device_descriptor(device_desc)
        .title("nannou web test")
        .size(WIDTH, HEIGHT)
        // .raw_event(raw_event)
        // .key_pressed(key_pressed)
        // .key_released(key_released)
        // .mouse_pressed(mouse_pressed)
        // .mouse_moved(mouse_moved)
        // .mouse_released(mouse_released)
        // .mouse_wheel(mouse_wheel)
        // .touch(touch)
        .view(view)
        .build_async()
        .await
        .unwrap();
}
