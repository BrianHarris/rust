use std::thread;
use std::thread::JoinHandle;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::Condvar;
use std::time;
use std::fmt;

extern crate rand;
use enum_primitive::FromPrimitive;

pub static mut TIME_FORK_PICKUP : i32 = 1000;
pub static mut TIME_FORK_PUTDOWN : i32 = 1000;
pub static mut TIME_EATING : i32 = 5000;
pub static mut TIME_THINKING : i32 = 5000;

enum_from_primitive! {
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum PhilosopherState {
    Thinking,
    WaitingForLeftFork,
    PickingUpLeftFork,
    WaitingForRightFork,
    PickingUpRightFork,
    PuttingDownForks,
    Eating,
}
}

impl fmt::Display for PhilosopherState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct DiningPhilosophers {
    philosopher_states : Arc<Vec<AtomicUsize>>,
    forks: Arc<Vec<AtomicIsize>>,
    forks_changed: Arc<(Mutex<()>, Condvar)>,
    thread_handles: Vec<JoinHandle<()>>,
}

fn random_wait(ms:i32)
{
    let half_ms = (ms / 2) + 1;
    let random_ms = half_ms + rand::random::<i32>() % half_ms;
    thread::sleep(time::Duration::from_millis(random_ms as u64));
}

impl DiningPhilosophers {

    pub fn new(count:usize) -> DiningPhilosophers
    {
        let mut dp = DiningPhilosophers {
            philosopher_states: Arc::new((0..count).map(|_i| AtomicUsize::new(PhilosopherState::Thinking as usize)).collect()),
            forks: Arc::new((0..count).map(|_i| AtomicIsize::new(-1)).collect()),
            forks_changed: Arc::new((Mutex::new(()), Condvar::new())),
            thread_handles: Vec::with_capacity(count),
        };

        for i in 0..count
        {
            let pstates = Arc::clone(&dp.philosopher_states);
            let forks = Arc::clone(&dp.forks);
            let forks_changed = Arc::clone(&dp.forks_changed);

            dp.thread_handles.push(thread::spawn(move || {
                let &(ref forks_changed_lock, ref forks_changed_cvar) = &*forks_changed;
                let left_fork = &forks[i];
                let right_fork = &forks[(i + 1) % count];
                let pstate = &pstates[i];
                loop {
                    pstate.store(PhilosopherState::WaitingForLeftFork as usize, Ordering::Relaxed);
                    while left_fork.compare_and_swap(-1, i as isize, Ordering::Relaxed) != -1 {
                        // Pickup fail, wait for the forks to change
                        let _m = forks_changed_cvar.wait(forks_changed_lock.lock().unwrap()).unwrap();
                    }

                    pstate.store(PhilosopherState::PickingUpLeftFork as usize, Ordering::Relaxed);
                    random_wait(unsafe{TIME_FORK_PICKUP});;

                    if right_fork.compare_and_swap(-1, i as isize, Ordering::Relaxed) != -1 {
                        // Pickup fail, put down the left fork
                        pstate.store(PhilosopherState::PuttingDownForks as usize, Ordering::Relaxed);
                        random_wait(unsafe {TIME_FORK_PUTDOWN});
                        left_fork.store(-1 as isize, Ordering::Relaxed);
                        forks_changed_cvar.notify_all();

                        // wait for forks to change
                        pstate.store(PhilosopherState::WaitingForRightFork as usize, Ordering::Relaxed);
                        while right_fork.compare_and_swap(-1, i as isize, Ordering::Relaxed) != -1 {
                            // Pickup fail, wait for the forks to change
                            let _m = forks_changed_cvar.wait(forks_changed_lock.lock().unwrap()).unwrap();
                        }

                        pstate.store(PhilosopherState::PickingUpRightFork as usize, Ordering::Relaxed);
                        random_wait(unsafe {TIME_FORK_PICKUP});

                        // attempt to pick up the left fork again
                        if left_fork.compare_and_swap(-1, i as isize, Ordering::Relaxed) != -1 {
                            // Pickup fail, put down the right fork and start all over again
                            pstate.store(PhilosopherState::PuttingDownForks as usize, Ordering::Relaxed);
                            random_wait(unsafe {TIME_FORK_PUTDOWN});
                            right_fork.store(-1 as isize, Ordering::Relaxed);
                            forks_changed_cvar.notify_all();
                            continue
                        } else {
                            pstate.store(PhilosopherState::PickingUpLeftFork as usize, Ordering::Relaxed);
                            random_wait(unsafe {TIME_FORK_PICKUP});
                        }
                    } else {
                        pstate.store(PhilosopherState::PickingUpRightFork as usize, Ordering::Relaxed);
                        random_wait(unsafe {TIME_FORK_PICKUP});
                    }

                    pstate.store(PhilosopherState::Eating as usize, Ordering::Relaxed);
                    random_wait(unsafe {TIME_EATING});

                    pstate.store(PhilosopherState::PuttingDownForks as usize, Ordering::Relaxed);
                    random_wait(unsafe {TIME_FORK_PUTDOWN});
                    left_fork.store(-1 as isize, Ordering::Relaxed);
                    right_fork.store(-1 as isize, Ordering::Relaxed);
                    forks_changed_cvar.notify_all();

                    pstate.store(PhilosopherState::Thinking as usize, Ordering::Relaxed);
                    random_wait(unsafe {TIME_THINKING});
                }
            }));
        }

        dp
    }

    pub fn wait(self)
    {
        for h in self.thread_handles {
            h.join().unwrap();
        }
    }

    pub fn get_state(&self, index:usize) -> PhilosopherState
    {
        PhilosopherState::from_usize(self.philosopher_states[index].load(Ordering::Relaxed)).unwrap()
    }

    pub fn get_fork(&self, index:usize) -> Option<usize>
    {
        let fork_owner = self.forks[index].load(Ordering::Relaxed);
        if fork_owner < 0 {
            None
        } else {
            Some(fork_owner as usize)
        }
    }
}
