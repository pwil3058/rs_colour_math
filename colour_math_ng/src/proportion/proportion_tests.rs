// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use super::*;
use num_traits_plus::assert_approx_eq;

#[test]
fn to_from_ufdf() {
    assert_eq!(UFDFraction::from(1.0_f64), UFDFraction::ONE);
    assert_eq!(f64::from(UFDFraction::ONE), 1.0);
    for f in &[0.0f64, 24.0, 0.8, 0.5, 2.0] {
        assert_approx_eq!(f64::from(UFDFraction::from(*f)), *f, 0.000_000_001);
    }
    assert_approx_eq!(UFDFraction::from(1.0_f32), UFDFraction::ONE, 0.000_000_001);
    assert_eq!(f32::from(UFDFraction::ONE), 1.0);
    for f in &[0.0f32, 24.0, 0.8, 0.5, 2.0] {
        assert_approx_eq!(f32::from(UFDFraction::from(*f)), *f, 0.000_000_001);
    }
    assert_eq!(UFDFraction::from(u8::MAX), UFDFraction::ONE);
    assert_eq!(u8::from(UFDFraction::ONE), u8::MAX);
    for u in 0_u8..u8::MAX {
        assert_eq!(u8::from(UFDFraction::from(u)), u);
    }
    assert_eq!(UFDFraction::from(u16::MAX), UFDFraction::ONE);
    assert_eq!(u16::from(UFDFraction::ONE), u16::MAX);
    for u in 0_u16..u16::MAX {
        assert_eq!(u16::from(UFDFraction::from(u)), u);
    }
}

#[test]
fn add_ufdf() {
    for [a, b] in &[[0.0f64, 1.0], [24.0, 0.5], [0.8, 0.5]] {
        let expected = UFDFraction::from(a + b);
        println!("{:?} | {:?} {:?}", a, b, expected);
        let result = UFDFraction::from(*a) + UFDFraction::from(*b);
        assert_eq!(result, expected);
        println!(
            "ADD{:?} == {:?} == {:?} == {:?}",
            result,
            expected,
            f64::from(result),
            f64::from(expected)
        );
        assert_approx_eq!(&f64::from(result), &(a + b), 0.000_000_001);
    }
}

#[test]
fn sub_ufdf() {
    for [a, b] in &[[2.0f64, 1.0], [24.0, 0.5], [0.8, 0.5]] {
        let expected = UFDFraction::from(a - b);
        println!("{:?} | {:?} {:?}", a, b, expected);
        let result = UFDFraction::from(*a) - UFDFraction::from(*b);
        println!(
            "SUB{:?} == {:?} == {:?} == {:?}",
            result,
            expected,
            f64::from(result),
            f64::from(expected)
        );
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_001);
    }
}

#[test]
fn div_ufdf() {
    for [a, b] in &[[2.0f64, 4.0], [24.0, 0.5], [0.8, 0.5]] {
        let expected = UFDFraction::from(a / b);
        println!("{:?} | {:?} {:?}", a, b, expected);
        let result = UFDFraction::from(*a) / UFDFraction::from(*b);
        println!(
            "DIV {:?} == {:?} == {:?} == {:?}",
            result,
            expected,
            f64::from(result),
            f64::from(expected)
        );
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
    }
}

#[test]
fn mul_ufdf() {
    for [a, b] in &[[2.0f64, 4.0], [24.0, 0.5], [0.8, 0.5]] {
        let expected = UFDFraction::from(a * b);
        println!("{:?} | {:?} {:?}", a, b, expected);
        let result = UFDFraction::from(*a) * UFDFraction::from(*b);
        println!(
            "DIV {:?} == {:?} == {:?} == {:?}",
            result,
            expected,
            f64::from(result),
            f64::from(expected)
        );
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
    }
}
