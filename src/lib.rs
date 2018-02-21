#![feature(allocator_api)]
#![feature(test)]
extern crate test;

mod owned_ptr;
pub mod vec;

#[cfg(test)]
mod tests {
    // use super::*;
    use vec::Vec;

    #[test]
    fn push_pop() {
        let mut v = Vec::new();

        const ELEM_NUM: usize = 32;
        let elems = 0..ELEM_NUM;

        for (i, e) in elems.clone().enumerate() {
            v.push(e);
            assert_eq!(v.len(), i + 1);
        }

        for (i, e) in elems.rev().enumerate() {
            let p = v.pop();
            assert_eq!(p, Some(e));
            assert_eq!(v.len(), ELEM_NUM - 1 - i);
        }

        assert!(v.pop().is_none());
    }

    #[test]
    fn deref_slice() {
        let mut v: Vec<i32> = Vec::new();
        assert!(v.is_empty());

        v.push(0);
        assert_eq!(v.len(), 1);
        assert_eq!(v.first(), Some(&0));
    }

    #[test]
    fn deref_mut_slice() {
        let mut v: Vec<i32> = Vec::new();

        v.push(0);
        v.push(1);
        v.reverse();

        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.pop(), Some(1));
    }
}
