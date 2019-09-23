// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{convert::From, ops::Index};

use crate::{ColourComponent, I_BLUE, I_GREEN, I_RED};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB<F: ColourComponent>([F; 3]);

impl<F: ColourComponent> RGB<F> {
    pub const RED: Self = Self([F::ONE, F::ZERO, F::ZERO]);
    pub const GREEN: Self = Self([F::ZERO, F::ONE, F::ZERO]);
    pub const BLUE: Self = Self([F::ZERO, F::ZERO, F::ONE]);

    pub const CYAN: Self = Self([F::ZERO, F::ONE, F::ONE]);
    pub const MAGENTA: Self = Self([F::ONE, F::ZERO, F::ONE]);
    pub const YELLOW: Self = Self([F::ONE, F::ONE, F::ZERO]);

    pub const WHITE: Self = Self([F::ONE, F::ONE, F::ONE]);
    pub const BLACK: Self = Self([F::ZERO, F::ZERO, F::ZERO]);

    pub const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    pub const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    pub const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];

    pub fn rgb(self) -> [F; 3] {
        self.0
    }

    pub fn rgba(self, alpha: F) -> [F; 4] {
        debug_assert!(alpha.is_proportion());
        [self.0[I_RED], self.0[I_GREEN], self.0[I_BLUE], alpha]
    }

    pub fn value(self) -> F {
        ((self.0[I_RED] + self.0[I_GREEN] + self.0[I_BLUE]) / F::THREE).min(F::ONE)
    }

    pub fn xy(self) -> (F, F) {
        let x = self.0[I_RED] + (self.0[I_GREEN] + self.0[I_BLUE]) * F::COS_120;
        let y = (self.0[I_GREEN] - self.0[I_BLUE]) * F::SIN_120;
        (x, y)
    }

    pub fn indices_value_order(self) -> [usize; 3] {
        if self[I_RED] >= self[I_GREEN] {
            if self[I_RED] >= self[I_BLUE] {
                if self[I_GREEN] >= self[I_BLUE] {
                    [I_RED, I_GREEN, I_BLUE]
                } else {
                    [I_RED, I_BLUE, I_GREEN]
                }
            } else {
                [I_BLUE, I_RED, I_GREEN]
            }
        } else if self[I_GREEN] >= self[I_BLUE] {
            if self[I_RED] >= self[I_BLUE] {
                [I_GREEN, I_RED, I_BLUE]
            } else {
                [I_GREEN, I_BLUE, I_RED]
            }
        } else {
            [I_BLUE, I_GREEN, I_RED]
        }
    }
}

impl<F: ColourComponent> Index<usize> for RGB<F> {
    type Output = F;

    fn index(&self, index: usize) -> &F {
        &self.0[index]
    }
}

impl<F: ColourComponent> From<[F; 3]> for RGB<F> {
    fn from(array: [F; 3]) -> Self {
        debug_assert!(array.iter().all(|x| (*x).is_proportion()));
        Self(array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_order() {
        assert_eq!(
            RGB::<f64>::WHITE.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::BLACK.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::RED.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::GREEN.indices_value_order(),
            [I_GREEN, I_RED, I_BLUE]
        );
        assert_eq!(
            RGB::<f64>::BLUE.indices_value_order(),
            [I_BLUE, I_RED, I_GREEN]
        );
        assert_eq!(
            RGB::<f64>::CYAN.indices_value_order(),
            [I_GREEN, I_BLUE, I_RED]
        );
        assert_eq!(
            RGB::<f64>::MAGENTA.indices_value_order(),
            [I_RED, I_BLUE, I_GREEN]
        );
        assert_eq!(
            RGB::<f64>::YELLOW.indices_value_order(),
            [I_RED, I_GREEN, I_BLUE]
        );
    }
}
