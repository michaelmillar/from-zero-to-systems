# graph

> Graph algorithms -- BFS, DFS, Dijkstra shortest paths, and topological sort.

## ELI5

A graph is just a collection of things (nodes) with connections between them (edges). Your social network is a graph. A road map is a graph. The list of packages that depend on each other in your project is a graph. Graph algorithms answer questions like: "what is the shortest route?", "in what order should I build these packages?", and "is there a path between these two points?"

## For the Educated Generalist

Four fundamental graph algorithms are implemented here:

**BFS (Breadth-First Search)** explores level by level using a queue. It finds the shortest path in an unweighted graph and is O(V + E). Used in social network "degrees of separation", web crawlers, and peer discovery in distributed systems.

**DFS (Depth-First Search)** explores as deep as possible before backtracking, using recursion (implicit stack). O(V + E). Used in cycle detection, topological ordering, and solving mazes.

**Dijkstra's algorithm** finds shortest paths in a weighted graph using a min-heap priority queue. O((V + E) log V). The foundational algorithm behind every GPS navigation system and network routing protocol (OSPF).

**Topological sort** (Kahn's algorithm) orders nodes such that every directed edge goes from earlier to later in the result. Only valid for Directed Acyclic Graphs (DAGs). O(V + E). Used in build systems (make, cargo, bazel), package managers, and task schedulers.

**Graph representation:** adjacency list (`HashMap<NodeId, Vec<(NodeId, weight)>>`). Better than adjacency matrix for sparse graphs (most real graphs are sparse) -- O(V + E) space vs O(V^2).

## Used in the wild

- **Cargo / npm / pip** -- topological sort orders the build/install of packages respecting dependencies
- **Google Maps / Waze** -- Dijkstra (or A*) on road networks with hundreds of millions of edges
- **Git** -- the commit DAG uses DFS for `git log`, `git bisect`, and reachability checks
- **LinkedIn** -- BFS to compute degrees of connection between users

## Run it

```bash
cargo run -p graph
```

## Use it as a library

```rust
use graph::Graph;

let mut g = Graph::new(4);
g.add_edge(0, 1, 5);
g.add_edge(0, 2, 1);
g.add_edge(2, 1, 2); // 0->2->1 costs 3

let dist = g.dijkstra(0);
assert_eq!(dist[&1], 3); // shortest path is via node 2
```

## Rust concepts covered

- **`BinaryHeap<Reverse<T>>`**: Rust's heap is a max-heap; wrap in `Reverse` to get a min-heap for Dijkstra
- **`HashMap` and `HashSet`**: standard library collections for adjacency lists and visited tracking
- **`VecDeque`**: O(1) push/pop from both ends; used as BFS queue
- **Recursion with mutable references**: DFS passes `&mut HashSet` and `&mut Vec` down the call stack

## Builds on

Standalone -- no earlier crates required.
