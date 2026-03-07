use sstable::{SSTableWriter, SSTableReader};
use std::io::{Cursor, Seek, SeekFrom};

fn main() {
    println!("=== SSTable Demo ===\n");

    let mut w = SSTableWriter::new();
    let data = [
        ("user:1001", "Alice,30"),
        ("user:1002", "Bob,25"),
        ("user:1003", "Charlie,35"),
        ("user:2001", "Diana,28"),
        ("user:2002", "Eve,31"),
    ];
    for (k, v) in &data {
        w.add(*k, *v);
    }

    let mut buf = Cursor::new(Vec::new());
    let size = w.finish(&mut buf).unwrap();
    println!("Written {} entries ({} bytes)\n", data.len(), size);

    buf.seek(SeekFrom::Start(0)).unwrap();
    let mut r = SSTableReader::open(buf).unwrap();

    println!("Point lookup 'user:1002': {:?}",
        r.get(b"user:1002").unwrap().map(|v| String::from_utf8(v).unwrap()));

    println!("\nRange scan user:1000-user:1999:");
    for (k, v) in r.scan(b"user:1000", b"user:1999").unwrap() {
        println!("  {} => {}", String::from_utf8(k).unwrap(), String::from_utf8(v).unwrap());
    }
}
