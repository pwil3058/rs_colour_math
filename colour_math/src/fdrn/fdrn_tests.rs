// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use num_traits_plus::assert_approx_eq;

use crate::{
    debug::ApproxEq,
    fdrn::{FDRNumber, Prop, UFDRNumber},
};

#[test]
fn sqrt_2() {
    assert_eq!(f64::from(FDRNumber::SQRT_2), std::f64::consts::SQRT_2);
    assert_eq!(f64::from(UFDRNumber::SQRT_2), std::f64::consts::SQRT_2);
}

#[test]
fn ufdrn_mul() {
    for [lhs, rhs] in &[
        [1.1_f64, 3.0],
        [0.0, 0.3],
        [1.024, 0.5],
        [0.18, 0.5],
        [3000.0, 0.02],
        [1.0, 1.001],
        [25.0, 5.0],
    ] {
        let expected = UFDRNumber::from(lhs * rhs);
        let result = UFDRNumber::from(*lhs) * UFDRNumber::from(*rhs);
        assert_approx_eq!(result, expected);
    }
}

#[test]
fn ufdrn_div() {
    for [lhs, rhs] in &[
        [1.1_f64, 3.0],
        [0.0, 0.3],
        [1.024, 0.5],
        [0.18, 0.5],
        [3000.0, 0.02],
        [1.0, 1.001],
        [25.0, 5.0],
    ] {
        let expected = UFDRNumber::from(lhs / rhs);
        let result = UFDRNumber::from(*lhs) / UFDRNumber::from(*rhs);
        assert_approx_eq!(result, expected, Prop(0x0000000000001000));
        let go_back = result * UFDRNumber::from(*rhs);
        assert_approx_eq!(go_back, UFDRNumber::from(*lhs), Prop(0x0000000000000002));
    }
}

#[test]
fn ufdrn_add() {
    for [a, b] in &[
        [0.0f64, 0.3],
        [0.024, 0.5],
        [0.18, 0.5],
        [0.5, 0.8],
        [1.5, 0.6],
    ] {
        let expected = UFDRNumber::from(a + b);
        let result = UFDRNumber::from(*a) + UFDRNumber::from(*b);
        assert_approx_eq!(result, expected, Prop(0x801));
        assert_approx_eq!(f64::from(result), &(a + b), Prop::from(0.000_000_01));
    }
}

#[test]
fn ufdrn_sub() {
    for [a, b] in &[
        [0.5f64, 0.3],
        [0.524, 0.5],
        [0.18, 0.15],
        [0.5, 0.08],
        [1.2, 1.1],
    ] {
        let expected = UFDRNumber::from(a - b);
        let result = UFDRNumber::from(*a) - UFDRNumber::from(*b);
        assert_approx_eq!(result, expected);
        assert_approx_eq!(f64::from(result), &(a - b), Prop::from(0.000_000_01));
    }
}

#[test]
fn ufdrn_div_u8() {
    for (a, b) in &[(0.9_f64, 3), (0.6, 2), (0.3, 2)] {
        let expected = UFDRNumber::from(*a / *b as f64);
        let result = UFDRNumber::from(*a) / *b;
        assert_approx_eq!(result, expected, Prop(0x0000000000005000));
        assert_approx_eq!(
            f64::from(result),
            &(a / *b as f64),
            Prop::from(0.000_000_01)
        );
    }
}

#[test]
fn composition() {
    for a in &[0.001_f64] {
        let prop = Prop::from(*a);
        for b in &[0.001_f64] {
            let second = Prop::from(*b);
            let left = UFDRNumber::THREE - (UFDRNumber::TWO - second) * prop;
            let right = UFDRNumber::THREE + second * prop - prop * 2;
            assert!(left > right);
            assert_approx_eq!(left, right);
        }
    }
}

#[test]
fn prop_mul() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.8, 0.5]] {
        let expected = Prop::from(a * b);
        let result = Prop::from(*a) * Prop::from(*b);
        assert_approx_eq!(result, expected, Prop(10));
        assert_approx_eq!(f64::from(result), &(a * b), Prop::from(0.000_000_01));
    }
}

#[test]
fn prop_mul_u8() {
    for (a, b) in &[(0.9_f64, 3_u8), (0.6, 2), (0.3, 2)] {
        let expected = UFDRNumber::from(*a * *b as f64);
        let result = Prop::from(*a) * *b;
        assert_approx_eq!(result, expected, Prop(0x801));
        assert_approx_eq!(
            f64::from(result),
            &(a * *b as f64),
            Prop::from(0.000_000_01)
        );
    }
}

#[test]
fn prop_div() {
    for [a, b] in &[[0.0f64, 0.3], [0.024, 0.5], [0.18, 0.5], [0.004, 0.6]] {
        let expected = Prop::from(a / b);
        let result = Prop::from(*a) / Prop::from(*b);
        assert_approx_eq!(result, expected);
        assert_approx_eq!(f64::from(result), (a / b));
        let go_back = result * Prop::from(*b);
        assert_approx_eq!(go_back, Prop::from(*a));
    }
}

#[test]
fn prop_add() {
    for [a, b] in &[
        [0.0f64, 0.3],
        [0.024, 0.5],
        [0.18, 0.5],
        [0.5, 0.8],
        [0.9, 0.04],
    ] {
        let expected = UFDRNumber::from(a + b);
        let result = Prop::from(*a) + Prop::from(*b);
        assert_approx_eq!(result, expected, Prop(0x0000000000005000));
        assert_approx_eq!(f64::from(result), &(a + b), Prop::from(0.000_000_01));
    }
}

#[test]
fn prop_sub() {
    for [a, b] in &[[0.5f64, 0.3], [0.524, 0.5], [0.18, 0.15], [0.5, 0.08]] {
        let expected = Prop::from(a - b);
        let result = Prop::from(*a) - Prop::from(*b);
        assert_approx_eq!(result, expected, Prop(0x101));
        assert_approx_eq!(f64::from(result), &(a - b), Prop::from(0.000_000_01));
    }
}

#[test]
fn from_fraction() {
    assert_eq!(Prop::from([1, 2]), Prop::HALF);
}

#[test]
fn from_unsigned() {
    assert_eq!(Prop::from(u32::MAX), Prop::ONE);
    assert_eq!(Prop::from(u8::MAX / 2), Prop::from([127, 255]));
}
