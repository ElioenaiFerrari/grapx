use std::collections::{HashMap, HashSet, VecDeque};

use actix_web::{post, web::Json, App, HttpResponse, HttpServer, Responder};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
enum GraphType {
    #[serde(rename(serialize = "directed", deserialize = "directed"))]
    Directed,
    #[serde(rename(serialize = "undirected", deserialize = "undirected"))]
    Undirected,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Graph<'a> {
    pub typ: GraphType,
    pub vertices: Vec<&'a String>,
    pub edges: HashMap<&'a String, Vec<&'a String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Response<'a> {
    pub graph: Graph<'a>,
    pub bfs_result: Vec<&'a String>,
    pub dfs_result: Vec<&'a String>,
    pub start: String,
}

impl<'a> Graph<'a> {
    pub fn new(typ: GraphType) -> Self {
        Graph {
            typ: typ,
            vertices: vec![],
            edges: HashMap::new(),
        }
    }

    fn add_vertex(&mut self, vertex: &'a String) {
        if !self.vertices.contains(&vertex) {
            self.vertices.push(vertex);
        }
    }

    pub fn add_edge(&mut self, origin: &'a String, destination: &'a String) {
        self.add_vertex(&origin);
        self.add_vertex(&destination);

        match self.typ {
            GraphType::Directed => {
                if !self.edges.contains_key(&origin) {
                    self.edges.insert(&origin, vec![]);
                }

                let entry = self.edges.entry(&origin).or_default();
                entry.push(destination);
            }
            GraphType::Undirected => {
                // Adiciona a relação em ambas as direções para grafos não direcionados
                if !self.edges.contains_key(&origin) {
                    self.edges.insert(&origin, vec![]);
                }
                if !self.edges.contains_key(&destination) {
                    self.edges.insert(&destination, vec![]);
                }

                let entry_origin = self.edges.entry(&origin).or_default();
                entry_origin.push(destination);

                let entry_dest = self.edges.entry(&destination).or_default();
                entry_dest.push(origin);
            }
        }
    }

    pub fn bfs(&self, start: &'a String) -> Vec<&'a String> {
        let mut visited = HashMap::new(); // Mapa para rastrear os nós visitados
        let mut result = vec![]; // Vetor para armazenar a ordem de visita

        // Fila para o algoritmo BFS
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while !queue.is_empty() {
            if let Some(node) = queue.pop_front() {
                if !visited.contains_key(&node) {
                    visited.insert(node, true);
                    result.push(node);

                    // Adiciona os vizinhos do nó à fila
                    if let Some(neighbors) = self.edges.get(&node) {
                        for neighbor in neighbors {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn dfs(&self, start: &'a String) -> Vec<&'a String> {
        let mut visited = HashSet::new(); // Conjunto para rastrear os nós visitados
        let mut result = vec![]; // Vetor para armazenar a ordem de visita
        let mut stack = VecDeque::new(); // Pilha para a busca em profundidade

        stack.push_back(start);

        while let Some(node) = stack.pop_back() {
            if !visited.contains(node) {
                visited.insert(node);
                result.push(node);

                if let Some(neighbors) = self.edges.get(node) {
                    // Inverte a ordem dos vizinhos para manter a ordem correta na pilha
                    for neighbor in neighbors.iter().rev() {
                        if !visited.contains(neighbor) {
                            stack.push_back(neighbor);
                        }
                    }
                }
            }
        }

        result
    }
}

#[derive(Debug, serde::Deserialize)]
struct Entry {
    graph_type: GraphType,
    edges: HashMap<String, Vec<String>>,
    start: String,
}

#[post("/")]
async fn analyse(params: Json<Entry>) -> impl Responder {
    let p = params.0;
    let mut graph = Graph::new(p.graph_type);

    for (origin, destinations) in &p.edges {
        for destination in destinations {
            graph.add_edge(&origin, &destination);
        }
    }

    let bfs = graph.bfs(&p.start);
    let dfs = graph.dfs(&p.start);

    let response = Response {
        bfs_result: bfs,
        dfs_result: dfs,
        graph: graph,
        start: p.start.to_owned(),
    };

    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(analyse))
        .bind(("0.0.0.0", 4000))?
        .run()
        .await
}
