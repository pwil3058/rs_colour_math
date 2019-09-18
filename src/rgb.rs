// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::ops::Index;

use num::traits::{Float, FloatConst, NumAssign, NumOps};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RGB<F: Float + FloatConst + NumAssign + NumOps>([F; 3]);

pub trait RGBConstants<F: Float + FloatConst + NumAssign + NumOps> {
    const RED: RGB<F>;
    const GREEN: RGB<F>;
    const BLUE: RGB<F>;

    const CYAN: RGB<F>;
    const MAGENTA: RGB<F>;
    const YELLOW: RGB<F>;

    const WHITE: RGB<F>;
    const BLACK: RGB<F>;

    const PRIMARIES: [RGB<F>; 3] = [Self::RED, Self::GREEN, Self::BLUE];
    const SECONDARIES: [RGB<F>; 3] = [Self::CYAN, Self::MAGENTA, Self::YELLOW];
    const GREYS: [RGB<F>; 2] = [Self::BLACK, Self::WHITE];
}

impl RGBConstants<f32> for RGB<f32> {
    const RED: RGB::<f32> = RGB([1.0, 0.0, 0.0]);
    const GREEN: RGB::<f32> = RGB([0.0, 1.0, 0.0]);
    const BLUE: RGB::<f32> = RGB([0.0, 0.0, 1.0]);

    const CYAN: RGB::<f32> = RGB([0.0, 1.0, 1.0]);
    const MAGENTA: RGB::<f32> = RGB([1.0, 0.0, 1.0]);
    const YELLOW: RGB::<f32> = RGB([1.0, 1.0, 0.0]);

    const WHITE: RGB::<f32> = RGB([1.0, 1.0, 1.0]);
    const BLACK: RGB::<f32> = RGB([0.0, 0.0, 0.0]);
}

impl RGBConstants<f64> for RGB<f64> {
    const RED: RGB::<f64> = RGB([1.0, 0.0, 0.0]);
    const GREEN: RGB::<f64> = RGB([0.0, 1.0, 0.0]);
    const BLUE: RGB::<f64> = RGB([0.0, 0.0, 1.0]);

    const CYAN: RGB::<f64> = RGB([0.0, 1.0, 1.0]);
    const MAGENTA: RGB::<f64> = RGB([1.0, 0.0, 1.0]);
    const YELLOW: RGB::<f64> = RGB([1.0, 1.0, 0.0]);

    const WHITE: RGB::<f64> = RGB([1.0, 1.0, 1.0]);
    const BLACK: RGB::<f64> = RGB([0.0, 0.0, 0.0]);
}

impl<F: Float + FloatConst + NumAssign + NumOps> RGB<F> {
    const I_RED: usize = 0;
    const I_GREEN: usize = 1;
    const I_BLUE: usize = 2;

    pub fn red_component(self) -> F {
        self.0[Self::I_RED]
    }

    pub fn green_component(self) -> F {
        self.0[Self::I_GREEN]
    }

    pub fn blue_component(self) -> F {
        self.0[Self::I_BLUE]
    }

    pub fn indices_value_order(self) -> [usize; 3] {
        if self.0[0] >= self.0[1] {
            if self.0[0] >= self.0[2] {
                if self.0[1] >= self.0[2] {
                    [0, 1, 2]
                } else {
                    [0, 2, 1]
                }
            } else {
                [2, 0, 1]
            }
        } else if self.0[1] >= self.0[2] {
            if self.0[0] >= self.0[2] {
                [1, 0, 2]
            } else {
                [1, 2, 0]
            }
        } else {
            [2, 1, 0]
        }
    }
}

impl<F: Float + FloatConst + NumAssign + NumOps> Index<usize> for RGB<F> {
    type Output = F;

    fn index(&self, index: usize) -> &F {
        &self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_order() {
        assert_eq!(RGB::<f64>::WHITE.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB::<f64>::BLACK.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB::<f64>::RED.indices_value_order(), [0, 1, 2]);
        assert_eq!(RGB::<f64>::GREEN.indices_value_order(), [1, 0, 2]);
        assert_eq!(RGB::<f64>::BLUE.indices_value_order(), [2, 0, 1]);
        assert_eq!(RGB::<f64>::CYAN.indices_value_order(), [1, 2, 0]);
        assert_eq!(RGB::<f64>::MAGENTA.indices_value_order(), [0, 2, 1]);
        assert_eq!(RGB::<f64>::YELLOW.indices_value_order(), [0, 1, 2]);
    }
}
