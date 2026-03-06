use graph::Graph;

fn main() {
    println!("=== Graph Algorithms Demo ===\n");

    // Package dependency graph (like cargo)
    // tokio -> mio, bytes
    // hyper -> tokio, bytes
    // axum  -> hyper, tokio
    let nodes = ["mio", "bytes", "tokio", "hyper", "axum"];
    let mut g = Graph::new(nodes.len());
    g.add_edge(2, 0, 1); // tokio -> mio
    g.add_edge(2, 1, 1); // tokio -> bytes
    g.add_edge(3, 2, 1); // hyper -> tokio
    g.add_edge(3, 1, 1); // hyper -> bytes
    g.add_edge(4, 3, 1); // axum  -> hyper
    g.add_edge(4, 2, 1); // axum  -> tokio

    println!("Dependency graph: axum -> hyper -> tokio -> mio/bytes\n");
    let topo = g.topological_sort().unwrap();
    print!("Build order (topo sort): ");
    println!("{}", topo.iter().map(|&i| nodes[i]).collect::<Vec<_>>().join(" -> "));

    println!("\n=== Dijkstra: City Road Network ===\n");
    //        2
    //   A ------- B
    //   |       / |
    // 4 |    3/   | 1
    //   |  /      |
    //   C -------- D
    //        5
    let cities = ["A", "B", "C", "D"];
    let mut road = Graph::new(4);
    road.add_undirected_edge(0, 1, 2); // A-B: 2
    road.add_undirected_edge(0, 2, 4); // A-C: 4
    road.add_undirected_edge(1, 2, 3); // B-C: 3
    road.add_undirected_edge(1, 3, 1); // B-D: 1
    road.add_undirected_edge(2, 3, 5); // C-D: 5

    let dist = road.dijkstra(0); // from A
    println!("Shortest distances from {}:", cities[0]);
    let mut sorted: Vec<_> = dist.iter().collect();
    sorted.sort_by_key(|(&n, _)| n);
    for (node, cost) in sorted {
        println!("  {} -> {}: {}", cities[0], cities[*node], cost);
    }
}
