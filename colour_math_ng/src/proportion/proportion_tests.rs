// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use super::*;
use num_traits_plus::assert_approx_eq;

#[test]
fn prop_mul() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.8, 0.5]] {
        let expected = Prop::from(a * b);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * b, expected);
        let result = Prop::from(*a) * Prop::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
    }
}

#[test]
fn prop_mul_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = Sum::from(*a * *b as f64);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * *b as f64, expected);
        let result = Prop::from(*a) * *b;
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a * *b as f64), 0.000_000_01);
    }
}

#[test]
fn prop_div() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5]] {
        let expected = Prop::from(a / b);
        println!("{:?} / {:?} = {:?} {:?}", a, b, a / b, expected);
        let result = Prop::from(*a) / Prop::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
    }
}

#[test]
fn prop_add() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5], [0.5, 0.8]] {
        let expected = Sum::from(a + b);
        println!("{:?} + {:?} = {:?} {:?}", a, b, a + b, expected);
        let result = Prop::from(*a) + Prop::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a + b), 0.000_000_01);
    }
}

#[test]
fn prop_sub() {
    for [a, b] in &[[0.5f64, 0.3], [0.524, 0.5], [0.18, 0.15], [0.5, 0.08]] {
        let expected = Prop::from(a - b);
        println!("{:?} - {:?} = {:?} {:?}", a, b, a - b, expected);
        let result = Prop::from(*a) - Prop::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
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
        let expected = Sum::from(a + b);
        println!("{:?} + {:?} = {:?} {:?}", a, b, a + b, expected);
        let result = Sum::from(*a) + Sum::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
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
        let expected = Sum::from(a - b);
        println!("{:?} - {:?} = {:?} {:?}", a, b, a - b, expected);
        let result = Sum::from(*a) - Sum::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a - b), 0.000_000_01);
    }
}

#[test]
fn sum_div() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5], [1.0, 1.001]] {
        let expected = Prop::from(a / b);
        println!("{:?} / {:?} = {:?} {:?}", a, b, a / b, expected);
        let result = Sum::from(*a) / Sum::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a / b), 0.000_000_01);
    }
}

#[test]
fn sum_div_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = Prop::from(*a / *b as f64);
        println!("{:?} / {:?} = {:?} : {:?}", a, b, a / *b as f64, expected);
        let result = Sum::from(*a) / *b;
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a / *b as f64), 0.000_000_01);
    }
}

#[test]
fn sum_mul_prop() {
    for [a, b] in &[[0.0f64, 0.3], [1.024, 0.5], [1.8, 0.5], [1.8, 0.99]] {
        let expected = Sum::from(a * b);
        println!("{:?} * {:?} = {:?} : {:?}", a, b, a * b, expected);
        let result = Sum::from(*a) * Prop::from(*b);
        assert_approx_eq!(result, expected, 0.000_000_001);
        assert_approx_eq!(f64::from(result), &(a * b), 0.000_000_01);
    }
}

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
    for [a, b] in &[[2.0f64, 4.0], [24.0, 0.5], [0.8, 0.5], [1.0, 1.001]] {
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
