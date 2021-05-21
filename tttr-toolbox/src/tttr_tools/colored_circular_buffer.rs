pub(super) struct CCircularBuffer {
    buffer: Vec<(u64, i32)>,
    pub head: i64,
}

impl CCircularBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_size),
            head: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, val: u64, ch: i32) {
        if self.len() < self.buffer.capacity() {
            self.buffer.push((val, ch));
        } else {
            let head = self.head;
            unsafe {
                let elem = self.buffer.get_unchecked_mut(head as usize);
                *elem = (val, ch);
            }
        }
        self.head = (self.head + 1) % (self.buffer.capacity() as i64);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn iter<'a>(&'a self) -> IterCCircularBuffer<'a> {
        IterCCircularBuffer {
            inner: self,
            pos: self.head - 1,
            oldest_idx: self.head - 1 - (self.len() as i64),
        }
    }
}

pub(super) struct IterCCircularBuffer<'a> {
    inner: &'a CCircularBuffer,
    pos: i64,
    oldest_idx: i64,
}

impl<'a> Iterator for IterCCircularBuffer<'a> {
    type Item = &'a (u64, i32);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos <= self.oldest_idx {
            None
        } else {
            self.pos -= 1;
            let wrap_around_idx = ((self.pos + 1) as usize) % self.inner.len();
            unsafe {
                let elem = &self.inner.buffer.get_unchecked(wrap_around_idx);
                Some(elem)
            }
        }
    }
}
