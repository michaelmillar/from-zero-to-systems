use memory_arena::Arena;
use std::time::Instant;

#[derive(Debug)]
#[allow(dead_code)]
struct Order {
    id:    u64,
    price: f64,
    qty:   u32,
    side:  u8,  // 0=buy, 1=sell
}

fn main() {
    const N: usize = 100_000;
    let arena_bytes = N * std::mem::size_of::<Order>() * 2; // 2× headroom

    println!("=== Memory Arena: HFT Order Book Allocator ===\n");
    println!("  Allocating {} orders ({} KB arena)\n", N, arena_bytes / 1024);

    // Arena allocation
    let mut arena = Arena::new(arena_bytes);
    let t0 = Instant::now();
    for i in 0..N {
        let order = arena.alloc::<Order>().expect("arena full");
        *order = Order { id: i as u64, price: 100.0 + i as f64 * 0.01, qty: 100, side: (i % 2) as u8 };
    }
    let arena_time = t0.elapsed();

    println!("  Arena:  {:>8} µs  ({} bytes used, {} remaining)",
        arena_time.as_micros(), arena.used(), arena.remaining());

    // Reset and reuse — zero allocation cost
    arena.reset();
    let t1 = Instant::now();
    for i in 0..N {
        let order = arena.alloc::<Order>().expect("arena full");
        *order = Order { id: i as u64, price: 99.0 + i as f64 * 0.01, qty: 200, side: (i % 2) as u8 };
    }
    let reuse_time = t1.elapsed();

    println!("  Reuse:  {:>8} µs  (same arena, reset to 0)", reuse_time.as_micros());
    println!("\n  Arena reset is O(1) — no deallocation, no GC pause.");
    println!("  This is why HFT systems and game engines use arenas.");
}
