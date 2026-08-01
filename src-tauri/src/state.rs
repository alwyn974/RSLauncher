use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Default)]
pub struct LaunchState {
    pub pid: Mutex<Option<u32>>,
    pub cancelled: AtomicBool,
    pub busy: AtomicBool,
}

impl LaunchState {
    pub fn reset_for_play(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
        *self.pid.lock().expect("pid mutex poisoned") = None;
    }

    pub fn set_pid(&self, pid: Option<u32>) {
        *self.pid.lock().expect("pid mutex poisoned") = pid;
    }

    pub fn take_pid(&self) -> Option<u32> {
        self.pid.lock().expect("pid mutex poisoned").take()
    }

    pub fn request_cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn set_busy(&self, busy: bool) {
        self.busy.store(busy, Ordering::SeqCst);
    }

    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }
}
