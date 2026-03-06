use hash_table::HashMap;

fn main() {
    println!("=== Hash Table Demo ===\n");

    let mut word_count: HashMap<&str, usize> = HashMap::new();
    let text = "the quick brown fox jumps over the lazy dog the fox";
    for word in text.split_whitespace() {
        let count = word_count.get(&word).copied().unwrap_or(0);
        word_count.insert(word, count + 1);
    }

    let mut words = ["the", "fox", "quick", "dog"];
    words.sort();
    println!("Word frequencies:");
    for word in &words {
        println!("  {:8} => {}", word, word_count.get(word).unwrap_or(&0));
    }

    println!("\nLoad factor after {} entries: {:.2}", word_count.len(), word_count.load_factor());

    println!("\n=== Robin Hood probing keeps max probe distance low ===");
    let mut m: HashMap<i32, i32> = HashMap::new();
    for i in 0..500 {
        m.insert(i, i);
    }
    println!("500 entries inserted, load factor: {:.2}", m.load_factor());
    println!("All 500 entries retrievable: {}", (0..500).all(|i| m.get(&i) == Some(&i)));
}
