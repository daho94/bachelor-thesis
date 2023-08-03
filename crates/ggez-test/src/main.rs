use ggez::{
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder, BLACK, WHITE},
    nalgebra as na, Context, GameResult,
};

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
        let mut g = Graph {
            nodes: Vec::new(),
            adjacency_list: Vec::new(),
            mesh: None,
        };
        g.add_node(200., 10.);
        g.add_node(200., 40.);
        g.add_node(200., 80.);
        g.add_node(400., 40.);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(0, 3);
        g.add_edge(2, 3);
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
        for node in &self.nodes {
            mesh_builder.circle(
                DrawMode::fill(),
                na::Point2::new(node.longitude, node.latitude),
                5.0, // Circle radius (adjust as needed)
                0.1, // Mesh tolerance (adjust as needed)
                BLACK,
            );
        }
        for (from, neighbors) in self.adjacency_list.iter().enumerate() {
            for &to in neighbors {
                let from_node = &self.nodes[from];
                let to_node = &self.nodes[to];
                mesh_builder.line(
                    &[
                        na::Point2::new(from_node.longitude, from_node.latitude),
                        na::Point2::new(to_node.longitude, to_node.latitude),
                    ],
                    1.0, // Line width (adjust as needed)
                    BLACK,
                );
            }
        }

        // Build the mesh and store it in the graph struct.
        self.mesh = Some(mesh_builder.build(ctx).unwrap());
    }

    // Render the graph.
    fn render(&self, ctx: &mut Context) -> GameResult<()> {
        if let Some(ref mesh) = self.mesh {
            graphics::draw(ctx, mesh, DrawParam::default())
        } else {
            Ok(())
        }
    }
}

// Implement the game event handler for the graph rendering.
struct Game {
    graph: Graph,
}

impl Game {
    fn new(ctx: &mut Context) -> Game {
        Game {
            graph: Graph::new(ctx),
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Add nodes or edges here if needed (e.g., on user input)
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, WHITE);
        self.graph.render(ctx)?;
        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("meshbatch", "ggez");
    // Setup ggez and create a window for rendering
    let (mut ctx, mut events_loop) = cb.build()?;

    let game = &mut Game::new(&mut ctx);
    event::run(&mut ctx, &mut events_loop, game)
}
