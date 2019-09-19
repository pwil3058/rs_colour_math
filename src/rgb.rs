// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::ops::Index;

use num::traits::Float;

pub trait RGBConstants: Sized {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    const CYAN: Self;
    const MAGENTA: Self;
    const YELLOW: Self;

    const WHITE: Self;
    const BLACK: Self;

    const PRIMARIES: [Self; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    const SECONDARIES: [Self; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    const GREYS: [Self; 2] = [Self::BLACK, Self::WHITE];
}

pub trait GRGB<T: PartialOrd + Copy + Sized>: Index<usize, Output=T> + Sized {
    const I_RED: usize = 0;
    const I_GREEN: usize = 1;
    const I_BLUE: usize = 2;

    fn red_component(self) -> T {
        self[Self::I_RED]
    }

    fn green_component(self) -> T {
        self[Self::I_GREEN]
    }

    fn blue_component(self) -> T {
        self[Self::I_BLUE]
    }

    fn components(self) -> [T; 3] {
        [self[Self::I_RED], self[Self::I_GREEN], self[Self::I_BLUE]]
    }

    fn indices_value_order(self) -> [usize; 3] {
        if self[0] >= self[1] {
            if self[0] >= self[2] {
                if self[1] >= self[2] {
                    [0, 1, 2]
                } else {
                    [0, 2, 1]
                }
            } else {
                [2, 0, 1]
            }
        } else if self[1] >= self[2] {
            if self[0] >= self[2] {
                [1, 0, 2]
            } else {
                [1, 2, 0]
            }
        } else {
            [2, 1, 0]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PRGB<F: Float + PartialOrd>([F; 3]);

impl RGBConstants for PRGB<f32> {
    const RED: PRGB::<f32> = PRGB([1.0, 0.0, 0.0]);
    const GREEN: PRGB::<f32> = PRGB([0.0, 1.0, 0.0]);
    const BLUE: PRGB::<f32> = PRGB([0.0, 0.0, 1.0]);

    const CYAN: PRGB::<f32> = PRGB([0.0, 1.0, 1.0]);
    const MAGENTA: PRGB::<f32> = PRGB([1.0, 0.0, 1.0]);
    const YELLOW: PRGB::<f32> = PRGB([1.0, 1.0, 0.0]);

    const WHITE: PRGB::<f32> = PRGB([1.0, 1.0, 1.0]);
    const BLACK: PRGB::<f32> = PRGB([0.0, 0.0, 0.0]);
}

impl RGBConstants for PRGB<f64> {
    const RED: PRGB::<f64> = PRGB([1.0, 0.0, 0.0]);
    const GREEN: PRGB::<f64> = PRGB([0.0, 1.0, 0.0]);
    const BLUE: PRGB::<f64> = PRGB([0.0, 0.0, 1.0]);

    const CYAN: PRGB::<f64> = PRGB([0.0, 1.0, 1.0]);
    const MAGENTA: PRGB::<f64> = PRGB([1.0, 0.0, 1.0]);
    const YELLOW: PRGB::<f64> = PRGB([1.0, 1.0, 0.0]);

    const WHITE: PRGB::<f64> = PRGB([1.0, 1.0, 1.0]);
    const BLACK: PRGB::<f64> = PRGB([0.0, 0.0, 0.0]);
}

impl<F: Float + PartialOrd> Index<usize> for PRGB<F> {
    type Output = F;

    fn index(&self, index: usize) -> &F {
        &self.0[index]
    }
}

impl<F: Float + PartialOrd> GRGB<F> for PRGB<F> {}

pub struct RGB24([u8; 3]);

impl RGBConstants for RGB24 {
    const RED: RGB24 = RGB24([255, 0, 0]);
    const GREEN: RGB24 = RGB24([0, 255, 0]);
    const BLUE: RGB24 = RGB24([0, 0, 255]);

    const CYAN: RGB24 = RGB24([0, 255, 255]);
    const MAGENTA: RGB24 = RGB24([255, 0, 255]);
    const YELLOW: RGB24 = RGB24([255, 255, 0]);

    const WHITE: RGB24 = RGB24([255, 255, 255]);
    const BLACK: RGB24 = RGB24([0, 0, 0]);
}

impl Index<usize> for RGB24 {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl GRGB<u8> for RGB24 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_order() {
        assert_eq!(PRGB::<f64>::WHITE.indices_value_order(), [0, 1, 2]);
        assert_eq!(PRGB::<f64>::BLACK.indices_value_order(), [0, 1, 2]);
        assert_eq!(PRGB::<f64>::RED.indices_value_order(), [0, 1, 2]);
        assert_eq!(PRGB::<f64>::GREEN.indices_value_order(), [1, 0, 2]);
        assert_eq!(PRGB::<f64>::BLUE.indices_value_order(), [2, 0, 1]);
        assert_eq!(PRGB::<f64>::CYAN.indices_value_order(), [1, 2, 0]);
        assert_eq!(PRGB::<f64>::MAGENTA.indices_value_order(), [0, 2, 1]);
        assert_eq!(PRGB::<f64>::YELLOW.indices_value_order(), [0, 1, 2]);

        assert_eq!(RGB24::WHITE.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB24::BLACK.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB24::RED.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB24::GREEN.indices_value_order(), [1, 0, 2]);
        assert_eq!(RGB24::BLUE.indices_value_order(), [2, 0, 1]);
        assert_eq!(RGB24::CYAN.indices_value_order(), [1, 2, 0]);
        assert_eq!(RGB24::MAGENTA.indices_value_order(), [0, 2, 1]);
        assert_eq!(RGB24::YELLOW.indices_value_order(), [0, 1, 2]);
    }
}
