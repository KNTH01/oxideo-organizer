use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Counter {
    all: AtomicUsize,
    media: AtomicUsize,
    processed: AtomicUsize,
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self {
            all: AtomicUsize::new(0),
            media: AtomicUsize::new(0),
            processed: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self, counter: Counters) {
        match counter {
            Counters::All => self.all.fetch_add(1, Ordering::SeqCst),
            Counters::Media => self.media.fetch_add(1, Ordering::SeqCst),
            Counters::Processed => self.processed.fetch_add(1, Ordering::SeqCst),
        };
    }

    pub fn get(&self, counter: Counters) -> usize {
        match counter {
            Counters::All => self.all.load(Ordering::SeqCst),
            Counters::Media => self.media.load(Ordering::SeqCst),
            Counters::Processed => self.processed.load(Ordering::SeqCst),
        }
    }
}

pub enum Counters {
    All,
    Media,
    Processed,
}
