// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use super::*;
use num_traits_plus::assert_approx_eq;

#[test]
fn prop_mul() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.8, 0.5]] {
        let expected = Prop::from(a * b);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * b, expected);
        let result = Prop::from(*a) * Prop::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 10);
        assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
    }
}

#[test]
fn prop_mul_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = UFDRNumber::from(*a * *b as f64);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * *b as f64, expected);
        let result = Prop::from(*a) * *b;
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x801);
        assert_approx_eq!(f64::from(result), &(a * *b as f64), 0.000_000_01);
    }
}

#[test]
fn prop_div() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5]] {
        let expected = Prop::from(a / b);
        println!("{:?} / {:?} = {:?} {:?}", a, b, a / b, expected);
        let result = Prop::from(*a) / Prop::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 10);
        assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
    }
}

#[test]
fn prop_add() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5], [0.5, 0.8]] {
        let expected = UFDRNumber::from(a + b);
        println!("{:?} + {:?} = {:?} {:?}", a, b, a + b, expected);
        let result = Prop::from(*a) + Prop::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x401);
        assert_approx_eq!(f64::from(result), &(a + b), 0.000_000_01);
    }
}

#[test]
fn prop_sub() {
    for [a, b] in &[[0.5f64, 0.3], [0.524, 0.5], [0.18, 0.15], [0.5, 0.08]] {
        let expected = Prop::from(a - b);
        println!("{:?} - {:?} = {:?} {:?}", a, b, a - b, expected);
        let result = Prop::from(*a) - Prop::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x101);
        assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_01);
    }
}

#[test]
fn sum_add() {
    for [a, b] in &[
        [0.0f64, 0.3],
        [0.024, 0.5],
        [0.18, 0.5],
        [0.5, 0.8],
        [1.5, 0.6],
    ] {
        let expected = UFDRNumber::from(a + b);
        println!(
            "{:?} + {:?} = {:?} {:?}",
            a,
            b,
            UFDRNumber::from(a + b),
            expected
        );
        let result = UFDRNumber::from(*a) + UFDRNumber::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x801);
        assert_approx_eq!(f64::from(result), &(a + b), 0.000_000_01);
    }
}

#[test]
fn sum_sub() {
    for [a, b] in &[
        [0.5f64, 0.3],
        [0.524, 0.5],
        [0.18, 0.15],
        [0.5, 0.08],
        [1.2, 1.1],
    ] {
        let expected = UFDRNumber::from(a - b);
        println!("{:?} - {:?} = {:?} {:?}", a, b, a - b, expected);
        let result = UFDRNumber::from(*a) - UFDRNumber::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 257);
        assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_01);
    }
}

#[test]
fn sum_div() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5], [1.0, 1.001]] {
        let expected = Prop::from(a / b);
        println!("{:?} / {:?} = {:?} {:?}", a, b, a / b, expected);
        let result = UFDRNumber::from(*a) / UFDRNumber::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x288);
        assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
    }
}

#[test]
fn sum_div_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = Prop::from(*a / *b as f64);
        println!("{:?} / {:?} = {:?} : {:?}", a, b, a / *b as f64, expected);
        let result = UFDRNumber::from(*a) / *b;
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x156);
        assert_approx_eq!(f64::from(result), &(a / *b as f64), 0.000_000_01);
    }
}

#[test]
fn sum_mul_prop() {
    for [a, b] in &[[0.0f64, 0.3], [1.024, 0.5], [1.8, 0.5], [1.8, 0.99]] {
        let expected = UFDRNumber::from(a * b);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * b, expected);
        let result = UFDRNumber::from(*a) * Prop::from(*b);
        println!("diff = {:#X}", result.abs_diff(&expected).0);
        assert_approx_eq!(result, expected, 0x10);
        assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
    }
}
