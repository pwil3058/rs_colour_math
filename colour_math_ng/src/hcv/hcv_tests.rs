// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use crate::hcv::*;

#[test]
fn default_hcv_is_black() {
    assert_eq!(HCV::default(), HCV::BLACK);
}

#[test]
fn create_hcv_consts_from_float() {
    assert_eq!(HCV::from(&RGB::<f64>::RED), HCV::RED);
    assert_eq!(HCV::from(&RGB::<f64>::GREEN), HCV::GREEN);
    assert_eq!(HCV::from(&RGB::<f64>::BLUE), HCV::BLUE);
    assert_eq!(HCV::from(&RGB::<f64>::CYAN), HCV::CYAN);
    assert_eq!(HCV::from(&RGB::<f64>::MAGENTA), HCV::MAGENTA);
    assert_eq!(HCV::from(&RGB::<f64>::YELLOW), HCV::YELLOW);
    assert_eq!(HCV::from(&RGB::<f64>::WHITE), HCV::WHITE);
    assert_eq!(HCV::from(&RGB::<f64>::BLACK), HCV::BLACK);
}

#[test]
fn create_hcv_consts_from_unsigned() {
    assert_eq!(HCV::from(&RGB::<u16>::RED), HCV::RED);
    assert_eq!(HCV::from(&RGB::<u16>::GREEN), HCV::GREEN);
    assert_eq!(HCV::from(&RGB::<u16>::BLUE), HCV::BLUE);
    assert_eq!(HCV::from(&RGB::<u16>::CYAN), HCV::CYAN);
    assert_eq!(HCV::from(&RGB::<u16>::MAGENTA), HCV::MAGENTA);
    assert_eq!(HCV::from(&RGB::<u16>::YELLOW), HCV::YELLOW);
    assert_eq!(HCV::from(&RGB::<u16>::WHITE), HCV::WHITE);
    assert_eq!(HCV::from(&RGB::<u16>::BLACK), HCV::BLACK);
}
