extern crate alloc;
use alloc::collections::vec_deque::VecDeque;
use std::sync::RwLock;

/// A very simple locking queue of bytes.
pub struct Queue {
    queue: RwLock<VecDeque<Vec<u8>>>,
    depth: u32,
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue {
    /// Return an initialized `Queue`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::from(vec![])),
            depth: 0,
        }
    }

    /// Re-queue bytes in b to the *front* of the queue.
    ///
    /// # Panics
    ///
    /// Panics if a lock cannot be attained.
    #[allow(clippy::expect_used)]
    pub fn requeue(
        &mut self,
        b: Vec<u8>,
    ) {
        self.queue
            .write()
            .expect("failed getting write lock to queue")
            .push_front(b);

        self.depth += 1;
    }

    /// Queue bytes in b to the *back* of the queue.
    ///
    /// # Panics
    ///
    ///  Panics if a lock cannot be attained.
    #[allow(clippy::expect_used)]
    pub fn enqueue(
        &mut self,
        b: Vec<u8>,
    ) {
        self.queue
            .write()
            .expect("failed getting write lock to queue")
            .push_back(b);

        self.depth += 1;
    }

    /// De-queue bytes from the queue.
    ///
    /// # Panics
    ///
    ///  Panics if a lock cannot be attained.
    #[allow(clippy::expect_used)]
    pub fn dequeue(&mut self) -> Vec<u8> {
        if self.depth == 0 {
            return vec![];
        }

        let b = self
            .queue
            .write()
            .expect("failed getting write lock to queue")
            .pop_front()
            .expect("unable to pop front from queue while dequeueing");

        self.depth -= 1;

        b
    }

    /// De-queue *all* bytes from the queue. As the queue is made up of a Vec of Vecs, this
    /// "flattens" all the Vecs into a single Vec of bytes.
    ///
    /// # Panics
    ///
    /// Panics if a lock cannot be attained.
    #[allow(dead_code)]
    #[allow(clippy::expect_used)]
    pub fn dequeue_all(&mut self) -> Vec<u8> {
        if self.depth == 0 {
            return vec![];
        }

        let mut queue_guard = self
            .queue
            .write()
            .expect("failed getting write lock to queue");

        let b = queue_guard.clone().into_iter().flatten().collect();

        queue_guard.clear();

        drop(queue_guard);

        self.depth = 0;

        b
    }

    /// Returns the current depth of the queue.
    pub const fn get_depth(&self) -> u32 {
        self.depth
    }
}
