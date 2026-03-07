use buffer_pool::{BufferPool, Disk};

fn main() {
    println!("=== Buffer Pool Demo ===\n");
    let mut disk = Disk::new();
    for _ in 0..8 { disk.alloc_page(); }

    let mut pool = BufferPool::new(3); // only 3 frames in memory
    println!("Pool capacity: 3 frames, Disk: 8 pages\n");

    // Write to page 0
    let p0 = pool.pin(0, &mut disk).unwrap();
    p0[0..5].copy_from_slice(b"hello");
    pool.unpin(0, true).unwrap();
    println!("Wrote 'hello' to page 0 (dirty)");

    // Access pages 1, 2, 3 -- page 0 may be evicted
    pool.pin(1, &mut disk).unwrap();
    pool.unpin(1, false).unwrap();
    pool.pin(2, &mut disk).unwrap();
    pool.unpin(2, false).unwrap();
    pool.pin(3, &mut disk).unwrap();
    pool.unpin(3, false).unwrap();

    println!("Accessed pages 1, 2, 3 (pool full -- page 0 may be evicted)\n");

    pool.flush_dirty(&mut disk);
    println!("Flushed dirty pages to disk");
    println!("Page 0 on disk: {:?}", std::str::from_utf8(&disk.read(0)[..5]).unwrap());
}
