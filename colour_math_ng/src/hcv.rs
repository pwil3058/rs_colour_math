// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::{Chroma, Hue, Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct HCV {
    pub(crate) hue_data: Option<Hue>,
    pub(crate) sum: Sum,
    pub(crate) chroma: Chroma,
}
