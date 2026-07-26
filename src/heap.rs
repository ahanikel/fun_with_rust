pub struct Heap {
    mem: Vec<u8>,
}

impl Heap {
    const MEM_SIZE: usize = 64*1024;
    #[cfg(test)]
    const GLOBAL_SIZE: u16 = 2;
    const METADATA_SIZE: u16 = 2;
    const MAX_FREE: u16 = 65532;
    fn read_u16(&self, idx: u16) -> u16 {
        let idx: usize = idx.into();
        u16::from_le_bytes([self.mem[idx], self.mem[idx+1]])
    }
    fn write_u16(&mut self, idx: u16, val: u16) {
        let idx: usize = idx.into();
        let val: [u8;2] = val.to_le_bytes();
        self.mem[idx]   = val[0];
        self.mem[idx+1] = val[1];
    }
    fn get_first_free(&self) -> u16 {
        self.read_u16(0)
    }
    fn set_first_free(&mut self, idx: u16) {
        self.write_u16(0, idx);
    }
    fn get_next_free(&self, idx: u16) -> u16 {
        self.read_u16(idx+2)
    }
    fn set_next_free(&mut self, idx: u16, next: u16) {
        self.write_u16(idx+2, next);
    }
    pub fn get_size(&self, idx: u16) -> u16 {
        self.read_u16(idx)
    }
    fn set_size(&mut self, idx: u16, size: u16) {
        self.write_u16(idx, size);
    }
    pub fn new() -> Heap {
        let mut ret = Heap { mem: vec![0; Self::MEM_SIZE] };
        ret.set_first_free(2);
        ret.set_size(2, Self::MAX_FREE);
        ret.set_next_free(2, 0);
        ret
    }
    pub fn malloc(&mut self, size: u16) -> Result<u16, OutOfMemoryError> {
        let size = if size < 2 { 2 } else { size };
        // find free block of sufficient size
        // 1. METADATA_SIZE for the new free block
        // 2. METADATA_SIZE for the new free block's next pointer
        let mut it = self.iter();
        let (idx, dont_split) = loop {
            match it.next() {
                // exact fit?
                Some(idx) if self.get_size(idx) == size =>
                    break (idx, true),
                Some(_idx) if Self::MAX_FREE < size =>
                    return Err(OutOfMemoryError {}),
                // enough room for splitting?
                Some(idx) if self.get_size(idx) >= size + Self::METADATA_SIZE * 2 =>
                    break (idx, false),
                Some(idx) if self.get_size(idx) > size =>
                    break (idx, true),
                Some(_) =>
                    continue,
                None => {
                    dbg!("Nothing found");
                    dbg!("Prev: {}, curr: {}, next: {}", it.previous_idx, it.current_idx, it.next_idx);
                    return Err(OutOfMemoryError {})},
            }
        };
        if dont_split {
            let previous_idx = it.previous_idx;
            let next_free = self.get_next_free(idx);
            match previous_idx {
                None => self.set_first_free(next_free),
                Some(prev) => self.set_next_free(prev, next_free),
            }
        } else {
            // split free block into used block and free block
            // remove old free block from the free list
            // insert new free block into the free list
            let previous_idx = it.previous_idx;
            let free_size = self.get_size(idx);
            let next_free = self.get_next_free(idx);
            let new_free_idx  = idx + Self::METADATA_SIZE + size;
            let new_free_size = free_size - size - Self::METADATA_SIZE;
            self.set_size(idx, size);
            self.set_size(new_free_idx, new_free_size);
            self.set_next_free(new_free_idx, next_free);
            match previous_idx {
                None => self.set_first_free(new_free_idx),
                Some(prev) => self.set_next_free(prev, new_free_idx),
            }
        }
        Ok(idx + Self::METADATA_SIZE)
    }
    pub fn free(&mut self, idx: u16) {
        let idx = idx - Self::METADATA_SIZE;
        let found = {
            let mut it = self.iter();
            loop {
                match it.next() {
                    Some(found_idx) if found_idx > idx =>
                        break (it.previous_idx, Some(found_idx)),
                    Some(_) =>
                        continue,
                    None =>
                        break (it.previous_idx, None),
                }
            }
        };
        match found {
            (previous_idx, Some(found_idx)) => {
                // insert before found_idx
                let size = self.get_size(idx);
                if idx + Self::METADATA_SIZE + size == found_idx {
                    // blocks are adjacent, merge them
                    let found_size = self.get_size(found_idx);
                    let found_next_free = self.get_next_free(found_idx);
                    let new_size = size + found_size + Self::METADATA_SIZE;
                    self.set_size(idx, new_size);
                    self.set_next_free(idx, found_next_free);
                } else {
                    self.set_next_free(idx, found_idx);
                }
                // insert after previous
                match previous_idx {
                    None => {
                        self.set_first_free(idx);
                    },
                    Some(prev) => {
                        let prev_size = self.get_size(prev);
                        if prev + Self::METADATA_SIZE + prev_size == idx {
                            // blocks are adjacent, merge them
                            let size = self.get_size(idx);
                            let next = self.get_next_free(idx);
                            let new_size = prev_size + Self::METADATA_SIZE + size;
                            self.set_size(prev, new_size);
                            self.set_next_free(prev, next);
                        } else {
                            self.set_next_free(prev, idx);
                        }
                    },
                }
            },
            (Some(prev), None) => {
                // insert at end
                let prev_size = self.get_size(prev);
                if prev + Self::METADATA_SIZE + prev_size == idx {
                    // blocks are adjacent, merge them
                    let size = self.get_size(idx);
                    let new_size = prev_size + Self::METADATA_SIZE + size;
                    self.set_size(prev, new_size);
                } else {
                    // blocks aren't adjacent, link them
                    self.set_next_free(prev, idx);
                    self.set_next_free(idx, 0);
                }
            },
            (None, None) => {
                self.set_first_free(idx);
                self.set_next_free(idx, 0);
            }
        }
    }
    fn iter(&self) -> FreeIterator<'_> {
        FreeIterator::new(self)
    }
}

#[derive(Debug)]
pub struct OutOfMemoryError {
}

struct FreeIterator<'a> {
    heap: &'a Heap,
    previous_idx: Option<u16>,
    current_idx: Option<u16>,
    next_idx: Option<u16>,
}

impl <'a> FreeIterator<'a> {
    fn new(heap: &'a Heap) -> FreeIterator<'a> {
        let previous_idx = None;
        let current_idx = None;
        let next_idx = heap.get_first_free();
        let next_idx =
            match next_idx {
                0 => None,
                _ => Some(next_idx),
            };
        FreeIterator { heap, previous_idx, current_idx, next_idx }
    }
}

impl<'a> Iterator for FreeIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_idx {
            None => {
                self.previous_idx = self.current_idx;
                self.current_idx = None;
                None
            },
            Some(idx) => {
                self.previous_idx = self.current_idx;
                self.current_idx = Some(idx);
                self.next_idx = {
                    let next = self.heap.get_next_free(idx);
                    if next == 0 {
                        None
                    } else {
                        Some(next)
                    }
                };
                Some(idx)
            }
        }
    }
}

mod test {
    #[test]
    fn test_too_big() {
        let mut heap = super::Heap::new();
        assert!(heap.malloc(65535).is_err());
        assert!(heap.malloc(65534).is_err());
        assert!(heap.malloc(65533).is_err());
    }
    #[test]
    fn test_max_alloc() {
        let mut heap = super::Heap::new();
        assert!(heap.malloc(65532).is_ok());
        assert!(heap.malloc(1).is_err());
    }
    #[test]
    fn test_allocs() {
        fn test_alloc(size: u16) {
            let mut heap = super::Heap::new();
            assert!(heap.malloc(size).is_ok());
        }
        for size in 1..=65532 {
            test_alloc(size);
        }
    }
    #[test]
    fn test_zero() {
        let mut heap = super::Heap::new();
        let ptr = heap.malloc(0);
        assert!(ptr.is_ok());
        assert_eq!(2, heap.get_size(ptr.unwrap()-2))
    }
    #[test]
    fn test_malloc() {
        let mut heap = super::Heap::new();
        // bytes 0 and 1 of the whole heap structure hold a pointer
        // to the first free block in the free list.
        assert_eq!(super::Heap::GLOBAL_SIZE, heap.read_u16(0));
        let next_free = heap.iter().next().unwrap();
        assert_eq!(super::Heap::GLOBAL_SIZE, next_free);
        assert_eq!(super::Heap::MAX_FREE, heap.get_size(next_free));
        let ptr = heap.malloc(1);
        assert!(ptr.is_ok());
        let ptr = ptr.unwrap();
        // note that the pointer which malloc returns is not the start
        // of the allocated block: the block starts two bytes earlier,
        // where bytes 0 and 1 contain the size of block usable by the
        // user
        assert_eq!(super::Heap::GLOBAL_SIZE + super::Heap::METADATA_SIZE,
                   ptr);
        // we always allocate a minimum of 2 bytes even if only 1 byte is requested.
        // we need those for additional free list metadata.
        assert_eq!(2, heap.get_size(ptr - super::Heap::METADATA_SIZE));
        let next_free = heap.iter().next().unwrap();
        assert_eq!(ptr + 2, next_free);
        assert_eq!(ptr + 2, heap.read_u16(0));
        assert_eq!(super::Heap::MAX_FREE - super::Heap::METADATA_SIZE - 2,
                   heap.get_size(next_free));

        let ptr2 = heap.malloc(1);
        assert!(ptr2.is_ok());
        let ptr2 = ptr2.unwrap();
        assert_eq!(ptr + 2 + super::Heap::METADATA_SIZE, ptr2);
        assert_eq!(2, heap.get_size(ptr2 - super::Heap::METADATA_SIZE));
        let next_free = heap.iter().next().unwrap();
        assert_eq!(super::Heap::MAX_FREE - 2 * (super::Heap::METADATA_SIZE + 2), heap.get_size(next_free));
    }
    #[test]
    fn test_to_the_max() {
        let mut heap = super::Heap::new();
        let mut allocs = Vec::new();
        // global metadata of the heap is 2 bytes;
        // a block including metadata is always at least 4 bytes,
        // therefore we can only do (65536 - 2) / 4 = 16383 allocations of 1 byte.
        // we could also say heap.malloc(2) here.
        for _ in 0..16383 {
            let alloc = heap.malloc(1);
            assert!(alloc.is_ok());
            let alloc = alloc.unwrap();
            assert_eq!(0, alloc % 4);
            allocs.push(alloc);
        }
        dbg!(allocs);
        assert!(heap.malloc(1).is_err());
    }
    #[test]
    fn test_to_the_max_and_free() {
        let mut heap = super::Heap::new();
        let mut allocs = Vec::new();
        // global metadata of the heap is 2 bytes;
        // a block including metadata is always at least 4 bytes,
        // therefore we can only do (65536 - 2) / 4 = 16383 allocations of 1 byte.
        // we could also say heap.malloc(2) here.
        for _ in 0..16383 {
            let alloc = heap.malloc(1);
            assert!(alloc.is_ok());
            let alloc = alloc.unwrap();
            assert_eq!(0, alloc % 4);
            allocs.push(alloc);
        }
        for alloc in allocs.iter().rev() {
            heap.free(*alloc);
        }
        let next_free = heap.iter().next().unwrap();
        assert_eq!(super::Heap::MAX_FREE, heap.get_size(next_free))
    }

    #[test]
    fn test_to_the_max_and_free_randomly() {
        let mut heap = super::Heap::new();
        let mut allocs = Vec::new();
        // global metadata of the heap is 2 bytes;
        // a block including metadata is always at least 4 bytes,
        // therefore we can only do (65536 - 2) / 4 = 16383 allocations of 1 byte.
        // we could also say heap.malloc(2) here.
        for _ in 0..16383 {
            let alloc = heap.malloc(1);
            assert!(alloc.is_ok());
            let alloc = alloc.unwrap();
            assert_eq!(0, alloc % 4);
            allocs.push(alloc);
        }
        while allocs.len() > 0 {
            let r = rand::random_range(0..allocs.len());
            let alloc = allocs.remove(r);
            heap.free(alloc);
        }
        let next_free = heap.iter().next().unwrap();
        assert_eq!(super::Heap::MAX_FREE, heap.get_size(next_free))
    }
}