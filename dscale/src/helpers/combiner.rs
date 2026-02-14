use std::usize;

/// Multiple values gathering (for example for quorums).
/// Combiner size is passed in compile time so compiler can place quorum on the stack.
pub struct Combiner<T: Sized, const K: usize> {
    quorum: [Option<T>; K],
    idx: usize,
}

impl<T: Sized, const K: usize> Combiner<T, K> {
    /// Constructs new quorum object
    pub fn new() -> Self {
        debug_assert!(K > 0, "Quorum threshold should be greater than zero");
        Self {
            quorum: [const { None }; K],
            idx: 0,
        }
    }

    /// Add a value to the quorum. Returns the complete quorum array when K values are collected.
    /// After returning a complete quorum each subsequent call will return None.
    pub fn combine(&mut self, value: T) -> Option<[T; K]> {
        self.quorum[self.idx] = Some(value);
        self.idx += 1;

        if self.idx == K {
            let mut result = [const { None }; K];
            std::mem::swap(&mut result, &mut self.quorum);
            // Unwraping is safe because we know all slots are filled
            Some(result.map(|opt| opt.unwrap()))
        } else {
            None
        }
    }
}
