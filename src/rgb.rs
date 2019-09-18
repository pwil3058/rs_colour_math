// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

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
}
