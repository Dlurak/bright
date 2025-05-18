use std::{iter::FusedIterator, num::NonZero};

pub struct AnimationIter {
    current: i32,
    desired: i32,
    change: NonZero<i32>,
}

impl Iterator for AnimationIter {
    type Item = (u16, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.desired {
            return None;
        }

        let remaining = self.desired - self.current;
        let last_step = remaining.abs() <= self.change.abs().get();

        if last_step {
            self.current = self.desired;
        } else {
            self.current += self.change.get();
        }

        Some((self.current as u16, last_step))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.desired - self.current;
        let count = remaining / self.change.get();

        let count_usize = usize::try_from(count)
            .expect("remaining and self.change must have the same sign => it is positive");
        if remaining % self.change.get() == 0 {
            (count_usize, Some(count_usize))
        } else {
            (count_usize + 1, Some(count_usize + 1))
        }
    }
}

impl FusedIterator for AnimationIter {}
impl ExactSizeIterator for AnimationIter {}

impl AnimationIter {
    pub fn new(current: u16, desired: u16, change: NonZero<i32>) -> Self {
        Self {
            current: current.into(),
            desired: desired.into(),
            change,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_iter() {
        let mut animation = AnimationIter::new(2, 10, NonZero::new(3).unwrap());
        assert_eq!(animation.len(), 3);
        assert_eq!(animation.next(), Some((5, false)));
        assert_eq!(animation.next(), Some((8, false)));
        assert_eq!(animation.next(), Some((10, true)));
        assert_eq!(animation.next(), None);
    }
}
