// ============================================================
//  YOUR CHALLENGE - implement a multi-level feedback queue
//  (MLFQ) scheduler.
//
//  MLFQ approximates optimal scheduling without knowing future
//  behaviour. It works by:
//    - Maintaining N priority queues (0 = highest priority).
//    - New processes start in queue 0.
//    - If a process uses its full time quantum, it is demoted
//      to the next lower queue (CPU-bound behaviour detected).
//    - If a process yields before its quantum expires, it stays
//      in (or is promoted to) the next higher queue (I/O-bound).
//    - Periodically, all processes are boosted to queue 0 to
//      prevent starvation (ageing).
//
//  This is the same fundamental algorithm behind Linux's O(1)
//  scheduler and a direct predecessor to CFS.
//
//  Note: in a real kernel, the PCB pool would use a slab
//  allocator (like the arena from 10-memory-arena). Here we use
//  a HashMap for clarity; the allocation pattern is identical.
// ============================================================

use std::collections::{HashMap, VecDeque};

pub const NUM_QUEUES: usize = 4;

/// Ticks per quantum for each queue level. Lower queue = longer quantum.
/// Queue 0: 10 ticks, Queue 1: 20, Queue 2: 40, Queue 3: 80.
pub const QUANTA: [u64; NUM_QUEUES] = [10, 20, 40, 80];

/// Age threshold: after this many ticks waiting without running,
/// promote process to queue 0.
pub const AGE_THRESHOLD: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Zombie,
}

#[derive(Debug, Clone)]
pub struct Pcb {
    pub pid: u32,
    pub name: &'static str,
    pub state: ProcessState,
    /// Current queue index (0 = highest priority).
    pub queue: usize,
    /// Ticks used in the current quantum.
    pub ticks_this_quantum: u64,
    /// Total CPU ticks consumed since spawn.
    pub total_cpu_ticks: u64,
    /// Ticks spent waiting in a ready queue (reset each time the process runs).
    pub wait_ticks: u64,
}

pub struct Scheduler {
    /// Ready queues indexed by priority (0 = highest).
    queues: [VecDeque<u32>; NUM_QUEUES],
    processes: HashMap<u32, Pcb>,
    /// Global tick counter used to trigger periodic ageing.
    tick: u64,
    next_pid: u32,
}

impl Scheduler {
    /// Create an empty scheduler.
    pub fn new() -> Self {
        todo!()
    }

    /// Spawn a new process and add it to queue 0 (highest priority).
    /// Returns the new PID.
    pub fn spawn(&mut self, name: &'static str) -> u32 {
        todo!()
    }

    /// Select the next process to run.
    /// Scans queues from highest to lowest priority and returns the
    /// PID at the front of the first non-empty queue.
    /// Returns None if all queues are empty.
    pub fn next_process(&mut self) -> Option<u32> {
        todo!()
    }

    /// Record one CPU tick for `pid`.
    ///
    /// `yielded_early`: true if the process voluntarily gave up the CPU
    /// before exhausting its quantum (I/O-bound behaviour).
    ///
    /// Behaviour:
    ///   - Increment ticks_this_quantum and total_cpu_ticks.
    ///   - If yielded_early: promote to max(queue - 1, 0), reset ticks_this_quantum,
    ///     move back to ready state in new queue.
    ///   - If quantum exhausted (ticks_this_quantum >= QUANTA[queue]):
    ///       demote to min(queue + 1, NUM_QUEUES - 1), reset ticks_this_quantum,
    ///       move back to ready state in new queue.
    ///   - Otherwise: move back to ready state in same queue.
    ///   - Increment wait_ticks for all OTHER ready processes.
    ///   - Increment self.tick; if tick % AGE_THRESHOLD == 0, run age_processes().
    pub fn tick(&mut self, pid: u32, yielded_early: bool) {
        todo!()
    }

    /// Block a process (e.g. waiting for I/O). Removes it from its ready queue.
    pub fn block(&mut self, pid: u32) {
        todo!()
    }

    /// Unblock a process. Adds it back to its current queue at the back.
    pub fn unblock(&mut self, pid: u32) {
        todo!()
    }

    /// Retrieve an immutable reference to a PCB by PID.
    pub fn get_pcb(&self, pid: u32) -> Option<&Pcb> {
        todo!()
    }

    /// Promote all processes that have waited longer than AGE_THRESHOLD to queue 0.
    /// Reset their wait_ticks to 0.
    fn age_processes(&mut self) {
        todo!()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
//  TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod empty_scheduler {
        use super::*;

        #[test]
        fn empty_scheduler_returns_none_from_next_process() {
            let mut sched = Scheduler::new();
            assert_eq!(sched.next_process(), None);
        }
    }

    mod priority_ordering {
        use super::*;

        #[test]
        fn higher_priority_process_runs_first() {
            let mut sched = Scheduler::new();
            let low = sched.spawn("low");
            // Exhaust low's first two quanta to demote it to queue 2
            for _ in 0..QUANTA[0] {
                sched.tick(low, false);
            }
            for _ in 0..QUANTA[1] {
                sched.tick(low, false);
            }
            // Now spawn a high-priority process
            let high = sched.spawn("high");
            assert_eq!(
                sched.next_process(),
                Some(high),
                "newly spawned (queue 0) process should preempt demoted process"
            );
        }

        #[test]
        fn cpu_bound_process_demotes_to_lower_queue() {
            let mut sched = Scheduler::new();
            let pid = sched.spawn("cpu-hog");
            let initial_queue = sched.get_pcb(pid).unwrap().queue;

            for _ in 0..QUANTA[initial_queue] {
                sched.tick(pid, false);
            }

            let new_queue = sched.get_pcb(pid).unwrap().queue;
            assert!(
                new_queue > initial_queue,
                "cpu-bound process should move to lower-priority queue after exhausting quantum"
            );
        }

        #[test]
        fn io_bound_process_is_promoted_after_early_yield() {
            let mut sched = Scheduler::new();
            let pid = sched.spawn("io-task");
            // Demote to queue 1 first
            for _ in 0..QUANTA[0] {
                sched.tick(pid, false);
            }
            let demoted_queue = sched.get_pcb(pid).unwrap().queue;
            assert!(demoted_queue > 0);

            // Yield early - should promote
            sched.tick(pid, true);
            let promoted_queue = sched.get_pcb(pid).unwrap().queue;
            assert!(
                promoted_queue < demoted_queue,
                "early-yielding process should be promoted toward queue 0"
            );
        }
    }

    mod ageing {
        use super::*;

        #[test]
        fn starved_process_is_aged_to_higher_queue() {
            let mut sched = Scheduler::new();
            // Demote victim to queue 3
            let victim = sched.spawn("victim");
            for _ in 0..(QUANTA[0] + QUANTA[1] + QUANTA[2]) {
                sched.tick(victim, false);
            }
            assert_eq!(sched.get_pcb(victim).unwrap().queue, 3);

            // Spin a different process long enough to trigger ageing
            let spinner = sched.spawn("spinner");
            for _ in 0..AGE_THRESHOLD {
                sched.tick(spinner, false);
            }

            assert_eq!(
                sched.get_pcb(victim).unwrap().queue,
                0,
                "victim waiting > AGE_THRESHOLD ticks should be boosted to queue 0"
            );
        }
    }

    mod block_unblock {
        use super::*;

        #[test]
        fn blocked_process_does_not_appear_in_next_process() {
            let mut sched = Scheduler::new();
            let pid = sched.spawn("worker");
            sched.block(pid);
            assert_eq!(sched.next_process(), None);
        }

        #[test]
        fn unblocked_process_becomes_schedulable_again() {
            let mut sched = Scheduler::new();
            let pid = sched.spawn("worker");
            sched.block(pid);
            sched.unblock(pid);
            assert_eq!(sched.next_process(), Some(pid));
        }
    }
}
