pub(super) struct CircularBuffer {
    buffer: Vec<u64>,
    pub head: i64,
}

impl CircularBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_size),
            head: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, val: u64) {
        if self.len() < self.buffer.capacity() {
            self.buffer.push(val);
        } else {
            let head = self.head;
            unsafe { 
                let elem = self.buffer.get_unchecked_mut(head as usize);
                *elem = val;
            }
        }
        self.head = (self.head + 1) % (self.buffer.capacity() as i64);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn iter<'a>(&'a self) -> IterCircularBuffer<'a> {
        IterCircularBuffer {
            inner: self,
            pos: self.head - 1,
            oldest_idx: self.head - 1 - (self.len() as i64),
        }
    }
}

pub(super) struct IterCircularBuffer<'a> {
    inner: &'a CircularBuffer,
    pos: i64,
    oldest_idx: i64,
}

impl<'a> Iterator for IterCircularBuffer<'a> {
    type Item = &'a u64;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos <= self.oldest_idx {
            None
        } else {
            self.pos -= 1;
            let wrap_around_idx = ((self.pos+1) as usize) % self.inner.len();
            unsafe {
                let elem = &self.inner.buffer.get_unchecked(wrap_around_idx);
                Some(elem)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn few_pushes() {
        let mut test_buff = CircularBuffer::new(16);
        test_buff.push(1);
        test_buff.push(3);
        assert_eq!(test_buff.buffer[0], 1);
        assert_eq!(test_buff.buffer[1], 3);
    }

    #[test]
    fn push_around() {
        let len: i64 = 16;
        let mut test_buff = CircularBuffer::new(len as usize);
        for i in 0..(2*len as usize) {
            test_buff.push(i as u64);
        }

        assert_eq!(test_buff.buffer[0], 16);
    }
}
