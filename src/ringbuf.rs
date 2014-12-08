use core::option::{Option, None, Some};
use core::intrinsics::*;
use core::ptr;

pub struct RingBuf<T> {
    pub head : uint,
    pub tail : uint,
    pub cap  : uint,
    pub buf  : *mut Option<T>
}

impl <T> RingBuf<T> {
    pub fn len(&self) -> uint {
        (self.tail + self.cap - self.head) % self.cap
    }

    pub fn enqueue(&mut self, elm: T) -> bool {
        let next_tail = (self.tail + 1) % self.cap;

        // Do not continue if we may overrung the head of the element
        // buffer.
        if next_tail == self.head {
            return false;
        }

        unsafe {
          let tail_elm = offset(self.buf as *const Option<T>, self.tail as int) as *mut Option<T>;
          *tail_elm = Some(elm);
        };
        self.tail = next_tail;
        return true;
    }

    pub unsafe fn dequeue(&mut self) -> Option<T> {
        let head_elm = offset(self.buf as *const Option<T>, self.head as int);
        let elm = ptr::read(head_elm);
        match elm {
            None => None,
            result@Some(_) => {
                *(head_elm as *mut Option<T>) = None;
                self.head = (self.head + 1) % self.cap;
                result
            }
        }
    }

    pub unsafe fn peek(&self) -> Option<T> {
        ptr::read(offset(self.buf as *const Option<T>, self.head as int))
    }
}

