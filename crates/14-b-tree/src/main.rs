use b_tree::BTree;

fn main() {
    println!("=== B-Tree Demo: Product Catalogue Index ===\n");
    let mut index: BTree<u32, &str> = BTree::new();

    let products = [
        (1042, "Laptop"), (2033, "Phone"), (1500, "Tablet"),
        (900, "Keyboard"), (3100, "Monitor"), (750, "Mouse"),
        (2500, "Headphones"), (1200, "Webcam"),
    ];
    for (id, name) in products {
        index.insert(id, name);
    }

    println!("Indexed {} products.\n", index.len());
    println!("Lookup product 1500: {:?}", index.get(&1500));
    println!("Lookup product 9999: {:?}\n", index.get(&9999));

    println!("Products with ID 1000-2000 (range scan):");
    for (id, name) in index.range(&1000, &2000) {
        println!("  {:4} -- {}", id, name);
    }

    println!("\nAll product IDs in sorted order:");
    println!("  {:?}", index.keys());
}
