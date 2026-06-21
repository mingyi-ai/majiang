use std::mem::MaybeUninit;

/// A fixed-capacity array-backed vector.
///
/// Stores up to `N` elements inline without heap allocation.
/// `T: Copy` enables safe `MaybeUninit` handling — uninitialized slots
/// are never read because the API constrains access to indices `0..len`.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct ArrayVec<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
    /// Empty array vec.
    pub fn new() -> Self {
        Self {
            data: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    /// Current number of initialized elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if no elements stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Remaining capacity.
    #[inline]
    pub fn remaining(&self) -> usize {
        N - self.len
    }

    /// Push one element at the end. Panics if full.
    pub fn push(&mut self, value: T) {
        assert!(self.len < N, "ArrayVec::push: capacity {N} exceeded");
        self.data[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Pop one element from the end. Returns `None` if empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.data[self.len].assume_init_read() })
        }
    }

    /// Reference to element at `index`. Panics if out of bounds.
    pub fn get(&self, index: usize) -> &T {
        assert!(
            index < self.len,
            "ArrayVec index {index} out of bounds (len {})",
            self.len
        );
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Mutable reference to element at `index`. Panics if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        assert!(
            index < self.len,
            "ArrayVec index {index} out of bounds (len {})",
            self.len
        );
        unsafe { self.data[index].assume_init_mut() }
    }

    /// Iterate over references to all initialized elements.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(|i| unsafe { self.data[i].assume_init_ref() })
    }

    /// Iterate over mutable references.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let len = self.len;
        let ptr = &mut self.data as *mut [MaybeUninit<T>; N];
        (0..len).map(move |i| unsafe { (*ptr)[i].assume_init_mut() })
    }

    /// Remove all elements.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Copy all elements from `other` into `self`, extending.
    pub fn extend_from(&mut self, other: &Self) {
        for i in 0..other.len() {
            self.push(unsafe { other.data[i].assume_init_read() });
        }
    }

    /// Position of the first element matching `pred`, or `None`.
    pub fn position(&self, mut pred: impl FnMut(&T) -> bool) -> Option<usize> {
        (0..self.len)
            .find(|&i| pred(unsafe { self.data[i].assume_init_ref() }))
    }

    /// Convert to a slice of initialized elements.
    /// The slice is valid for the lifetime of the borrow.
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.data.as_ptr() as *const T,
                    self.len,
                )
            }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.data.as_mut_ptr() as *mut T,
                    self.len,
                )
            }
        }
    }
}

impl<T: Copy, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> std::ops::Index<usize> for ArrayVec<T, N> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        self.get(index)
    }
}

impl<T: Copy, const N: usize> std::ops::IndexMut<usize> for ArrayVec<T, N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index)
    }
}
