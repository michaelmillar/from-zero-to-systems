use slotted_page::SlottedPage;

fn main() {
    println!("=== Slotted Page Demo ===\n");
    let mut page = SlottedPage::new();

    let rows = [
        b"Alice,30,Engineer".as_ref(),
        b"Bob,25,Designer".as_ref(),
        b"Charlie,35,Manager".as_ref(),
    ];

    let ids: Vec<_> = rows.iter().map(|r| page.insert(r).unwrap()).collect();
    println!("Inserted {} records. Free space: {} bytes", ids.len(), page.free_space());

    for id in &ids {
        println!("  slot {}: {:?}", id, std::str::from_utf8(page.get(*id).unwrap()).unwrap());
    }

    println!("\nDeleting slot 1 (Bob)...");
    page.delete(ids[1]).unwrap();

    println!("Compacting page...");
    page.compact();
    println!("Free space after compact: {} bytes", page.free_space());

    println!("\nAfter compaction:");
    println!("  slot 0: {:?}", std::str::from_utf8(page.get(ids[0]).unwrap()).unwrap());
    println!("  slot 2: {:?}", std::str::from_utf8(page.get(ids[2]).unwrap()).unwrap());
}
