// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::beigui::FDRNumber;

#[test]
fn div_by_u8() {
    assert_eq!(FDRNumber::ONE / 2, FDRNumber(u64::MAX as i128 / 2));
    assert_eq!(FDRNumber::ONE / 3, FDRNumber(u64::MAX as i128 / 3));
}
