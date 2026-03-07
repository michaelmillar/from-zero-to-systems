use skip_list::SkipList;

fn main() {
    println!("=== Skip List Demo: LSM MemTable ===\n");

    let mut memtable: SkipList<String, String> = SkipList::new();

    let writes = [
        ("user:alice", "age=30"),
        ("user:bob",   "age=25"),
        ("user:carol", "age=35"),
        ("user:alice", "age=31"), // update
        ("txn:1001",   "amount=500"),
        ("txn:1002",   "amount=200"),
    ];

    for (k, v) in &writes {
        memtable.insert(k.to_string(), v.to_string());
        println!("  write: {} = {}", k, v);
    }

    println!("\nMemTable size: {} entries\n", memtable.len());

    println!("Point lookup 'user:alice': {:?}", memtable.get(&"user:alice".to_string()));
    println!("Point lookup 'user:dave':  {:?}", memtable.get(&"user:dave".to_string()));

    println!("\nRange scan user:* :");
    for (k, v) in memtable.range(&"user:".to_string(), &"user:~".to_string()) {
        println!("  {} = {}", k, v);
    }

    println!("\n(In a real LSM-tree, this MemTable would be flushed to an SSTable -- see crate 28)");
}
