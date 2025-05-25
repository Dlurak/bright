pub mod easing;

use crate::animation::easing::Easing;
use std::{iter::FusedIterator, num::NonZero};

pub struct AnimationIter<T: Easing> {
    current: u16,
    frame_count: usize,

    // constant
    desired: u16,
    max: u16,
    easing: T,
}

impl<T: Easing> Iterator for AnimationIter<T> {
    type Item = (u16, bool);

    fn next(&mut self) -> Option<Self::Item> {
        match self.frame_count {
            0 => return None,
            1 => {
                self.current = self.desired;
                self.frame_count -= 1;
                return Some((self.desired, true));
            }
            _ => {}
        }

        let current_actual = f64::from(self.current) / f64::from(self.max);
        let desired_actual = f64::from(self.desired) / f64::from(self.max);

        let current_userfacing = self.easing.from_actual(current_actual);

        let user_step = (self.easing.from_actual(desired_actual) - current_userfacing)
            / self.frame_count as f64;

        let new_actual = self.easing.to_actual(current_userfacing + user_step);

        self.current = (new_actual * f64::from(self.max)).round() as u16;
        self.frame_count -= 1;

        Some((self.current, false))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.frame_count, Some(self.frame_count))
    }
}

impl<T: Easing> FusedIterator for AnimationIter<T> {}
impl<T: Easing> ExactSizeIterator for AnimationIter<T> {}

impl<T: Easing> AnimationIter<T> {
    pub fn new(
        (current, desired): (u16, u16),
        max: u16,
        frame_count: NonZero<usize>,
        easing: T,
    ) -> Self {
        Self {
            current,
            frame_count: frame_count.get(),
            desired,
            max,
            easing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::easing::EasingKind;

    #[test]
    fn test_animation_iter_linear() {
        let mut animation =
            AnimationIter::new((2, 10), 10, NonZero::new(3).unwrap(), EasingKind::Linear);
        assert_eq!(animation.len(), 3);
        assert_eq!(animation.next(), Some((5, false)));
        assert_eq!(animation.next(), Some((8, false)));
        assert_eq!(animation.next(), Some((10, true)));
        assert_eq!(animation.next(), None);
    }
}
