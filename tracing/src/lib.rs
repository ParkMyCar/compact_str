use parking_lot::{
    FairMutex,
    const_fair_mutex,
};
use std::alloc::{GlobalAlloc, System};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub enum Event {
    Alloc { addr: usize, size: usize },
    Freed { addr: usize, size: usize },
}

impl Event {
    pub fn delta(&self) -> isize {
        match self {
            Self::Alloc { size, .. } => *size as isize,
            Self::Freed { size, .. } => (*size as isize) * -1,
        }
    }
}

pub struct TracingAllocator {
    pub log: FairMutex<Vec<Event>>,
    pub enabled: AtomicBool,
}

impl TracingAllocator {
    pub const fn new() -> Self {
        Self {
            log: const_fair_mutex(Vec::new()),
            enabled: AtomicBool::new(false),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn log_event(&self, event: Event) {
        if self.is_enabled() {
            self.disable();
            let mut log = self.log.lock();
            log.push(event);
            self.enable();
        }
    }

    pub fn events(&self) -> Vec<Event> {
        let log = self.log.lock();
        log.clone()
    }
}

unsafe impl GlobalAlloc for TracingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let res = System.alloc(layout);
        self.log_event(Event::Alloc {
            addr: res as _,
            size: layout.size()
        });
        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.log_event(Event::Freed {
            addr: ptr as _,
            size: layout.size()
        });
        System.dealloc(ptr, layout)
    }
}
