use std::{collections::HashMap, path::Path};

use ggez::{
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, Transform},
    input::keyboard,
    mint::{self, Point2, Vector2},
    Context, GameResult,
};
use ggez::{graphics::Canvas, input::keyboard::KeyCode};
use osm_reader::{Arc, RoadGraph};

// Define a simple struct to represent nodes with latitude and longitude coordinates.
struct Node {
    latitude: f32,
    longitude: f32,
}

// Define a struct to represent the graph and its mesh.
struct Graph {
    nodes: Vec<Node>,
    adjacency_list: Vec<Vec<usize>>,
    mesh: Option<Mesh>,
}

impl Graph {
    // Create a new empty graph.
    fn new(ctx: &mut Context) -> Self {
        // let road_graph = RoadGraph::from_pbf(Path::new("../osm_reader/data/vaterstetten.osm.pbf"))
        let road_graph =
            RoadGraph::from_pbf(Path::new("../osm_reader/data/saarland_pp2.osm.pbf")).unwrap();

        let mut g = Graph {
            nodes: Vec::new(),
            adjacency_list: Vec::new(),
            mesh: None,
        };

        // Obtain screen dimensions
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        // let screen_width = graphics::screen_coordinates(ctx).w;
        // let screen_height = graphics::screen_coordinates(ctx).h;

        let mut id_map = HashMap::new();

        // Get MIN_LONGITUDE, MAX_LONGITUDE
        let mut min_lon = f32::MAX;
        let mut min_lat = f32::MAX;
        let mut max_lon = f32::MIN;
        let mut max_lat = f32::MIN;

        for (_, &[lat, lon]) in road_graph.get_nodes().iter() {
            min_lon = min_lon.min(lon as f32);
            max_lon = max_lon.max(lon as f32);
            min_lat = min_lat.min(lat as f32);
            max_lat = max_lat.max(lat as f32);
        }

        dbg!(min_lon);
        dbg!(max_lon);
        dbg!(min_lat);
        dbg!(max_lat);

        for (idx, (id, &[lat, lon])) in road_graph.get_nodes().iter().enumerate() {
            let screen_x = (lon as f32 - min_lon) / (max_lon - min_lon) * screen_width;
            let screen_y =
                screen_height - (lat as f32 - min_lat) / (max_lat - min_lat) * screen_height;
            g.add_node(screen_x, screen_y);
            id_map.insert(id, idx);
        }

        for &Arc { source, target, .. } in road_graph.get_arcs() {
            let src_idx = id_map.get(&source).unwrap();
            let dst_idx = id_map.get(&target).unwrap();
            g.add_edge(*src_idx, *dst_idx);
        }

        g.create_mesh(ctx);
        g
    }

    // Add a node to the graph.
    fn add_node(&mut self, latitude: f32, longitude: f32) {
        self.nodes.push(Node {
            latitude,
            longitude,
        });
        self.adjacency_list.push(Vec::new());
        // Recreate the mesh when a new node is added.
        // self.create_mesh();
    }

    // Add a directed edge between two nodes.
    fn add_edge(&mut self, from: usize, to: usize) {
        self.adjacency_list[from].push(to);
        // Recreate the mesh when a new edge is added.
        // self.create_mesh();
    }

    // Create or recreate the mesh based on the current graph state.
    fn create_mesh(&mut self, ctx: &mut Context) {
        let mut mesh_builder = MeshBuilder::new();
        // Build the mesh here based on the nodes and edges in the graph.
        // You can use `mesh_builder.line()` to add line segments for edges
        // and `mesh_builder.circle()` to add nodes.
        // For simplicity, let's just draw circles for the nodes and lines for the edges.
        // for node in &self.nodes {
        //     mesh_builder.circle(
        //         DrawMode::fill(),
        //         na::Point2::new(node.longitude, node.latitude),
        //         1.0, // Circle radius (adjust as needed)
        //         0.1, // Mesh tolerance (adjust as needed)
        //         BLACK,
        //     );
        // }
        for (from, neighbors) in self.adjacency_list.iter().enumerate() {
            for &to in neighbors {
                let from_node = &self.nodes[from];
                let to_node = &self.nodes[to];
                let _ = mesh_builder.line(
                    &[
                        Vec2::new(from_node.longitude, from_node.latitude),
                        Vec2::new(to_node.longitude, to_node.latitude),
                    ],
                    1.0, // Line width (adjust as needed)
                    Color::BLACK,
                );
            }
        }

        // Build the mesh and store it in the graph struct.
        self.mesh = Some(Mesh::from_data(ctx, mesh_builder.build()));
    }

    // Render the graph.
    fn render(&self, canvas: &mut Canvas, params: DrawParam) -> GameResult {
        // let mut params = DrawParam::default();
        // let mut view = Rect::one();
        // params.src = view;
        // params.rotation = 180./360.;
        // params.scale = Vector2 { x: 2.5, y: 2.5 };
        if let Some(ref mesh) = self.mesh {
            // graphics::draw(ctx, mesh, params)
            canvas.draw(mesh, params);
            Ok(())
        } else {
            Ok(())
        }
    }
}
struct Camera {
    x: f32,
    y: f32,
    zoom: f32,
    speed: f32,
    zoom_speed: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            speed: 5.0,
            zoom_speed: 0.1,
        }
    }
}

// Implement the game event handler for the graph rendering.
struct Game {
    graph: Graph,
    camera: Camera,
}

impl Game {
    fn new(ctx: &mut Context) -> Game {
        Game {
            graph: Graph::new(ctx),
            camera: Camera::default(),
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let k_ctx = &ctx.keyboard;
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.camera.y -= self.camera.speed / self.camera.zoom;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.camera.y += self.camera.speed / self.camera.zoom;
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.camera.x -= self.camera.speed / self.camera.zoom;
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.camera.x += self.camera.speed / self.camera.zoom;
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.camera.zoom += self.camera.zoom_speed;
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.camera.zoom -= self.camera.zoom_speed;
        }
        if k_ctx.is_key_pressed(KeyCode::R) {
            self.camera = Camera::default();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        dbg!(ctx.time.fps());

        // Set the camera view before drawing anything
        // let zoom_matrix = ggez::graphics::Transform::scale(self.camera.zoom, self.camera.zoom);
        // let trans_matrix = ggez::graphics::Transform::translate(-self.camera.x, -self.camera.y);
        // ggez::graphics::set_transform(ctx, zoom_matrix * trans_matrix)?;
        let transform = Transform::Values {
            dest: mint::Point2 {
                x: -self.camera.x,
                y: -self.camera.y,
            },
            rotation: 0.0,
            scale: mint::Vector2 {
                x: self.camera.zoom,
                y: self.camera.zoom,
            },
            offset: mint::Point2 { x: 0.0, y: 0.0 },
        };

        let params = DrawParam::default()
            // .scale(Vec2::new(self.zoom, self.zoom))
            // .dest(self.pos);
            .transform(transform.to_bare_matrix());

        self.graph.render(&mut canvas, params)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("meshbatch", "ggez");
    // Setup ggez and create a window for rendering
    let (mut ctx, events_loop) = cb.build()?;

    let game = Game::new(&mut ctx);
    event::run(ctx, events_loop, game)
}
