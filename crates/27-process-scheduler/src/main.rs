fn main() {
    use process_scheduler::Scheduler;

    let mut sched = Scheduler::new();
    let web   = sched.spawn("web-handler");
    let batch = sched.spawn("batch-job");
    let _io   = sched.spawn("disk-io");

    for t in 0..30u64 {
        if let Some(pid) = sched.next_process() {
            let yielded = pid == web || t % 3 == 0;
            sched.tick(pid, yielded);
        }
    }

    for &pid in &[web, batch] {
        if let Some(pcb) = sched.get_pcb(pid) {
            println!("{}: queue={} total_cpu={}", pcb.name, pcb.queue, pcb.total_cpu_ticks);
        }
    }
}
