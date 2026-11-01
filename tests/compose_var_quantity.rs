use std::assert_matches;

use serde::{Deserialize, Serialize};
use var_quantity::uom::si::f64::*;
use var_quantity::uom::si::{
    area::square_meter, length::meter, ratio::ratio, reciprocal_length::reciprocal_meter,
    volume::cubic_meter,
};
use var_quantity::uom::typenum::P2;
use var_quantity::*;

#[test]
fn test_constants() {
    {
        let res: VarQuantity<Length> = VarQuantity::from(Length::new::<meter>(1.0))
            + VarQuantity::from(Length::new::<meter>(2.0));
        assert_eq!(res.eval(&[]).get::<meter>(), 3.0);

        let res: VarQuantity<Length> = res + VarQuantity::from(Length::new::<meter>(3.0));
        assert_eq!(res.eval(&[]).get::<meter>(), 6.0);
    }
    {
        let res: VarQuantity<Length> = VarQuantity::from(Length::new::<meter>(6.0))
            - VarQuantity::from(Length::new::<meter>(5.0));
        assert_eq!(res.eval(&[]).get::<meter>(), 1.0);

        let res: VarQuantity<Length> = res - VarQuantity::from(Length::new::<meter>(3.0));
        assert_eq!(res.eval(&[]).get::<meter>(), -2.0);
    }
    {
        let res = VarQuantity::from(Length::new::<meter>(2.0))
            * VarQuantity::from(Length::new::<meter>(3.0));
        assert_eq!(res.eval(&[]).get::<square_meter>(), 6.0);

        let prod: VarQuantity<Volume> = res * VarQuantity::from(Length::new::<meter>(4.0));
        assert_eq!(prod.eval(&[]).get::<cubic_meter>(), 24.0);
    }
    {
        let res = VarQuantity::from(Volume::new::<cubic_meter>(8.0))
            / VarQuantity::from(Length::new::<meter>(2.0));
        assert_eq!(res.eval(&[]).get::<square_meter>(), 4.0);

        let prod: VarQuantity<Length> = res / VarQuantity::from(Length::new::<meter>(4.0));
        assert_eq!(prod.eval(&[]).get::<meter>(), 1.0);
    }
    {
        let squared = VarQuantity::from(2.0).powi(2);
        assert_eq!(squared.eval(&[]), 4.0);

        let squared = VarQuantity::from(Length::new::<meter>(4.0)).powi(P2::new());
        assert_eq!(squared.eval(&[]).get::<square_meter>(), 16.0);
    }
    {
        let absolute = VarQuantity::from(-2.0).abs();
        assert_eq!(absolute.eval(&[]), 2.0);

        let absolute = VarQuantity::from(Length::new::<meter>(-2.0)).abs();
        assert_eq!(absolute.eval(&[]).get::<meter>(), 2.0);
    }
    {
        let sum = VarQuantity::from(-2.0) - VarQuantity::from(8.0);
        assert_eq!(sum.eval(&[]), -10.0);
        assert_eq!(sum.abs().eval(&[]), 10.0);
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Constant(DynQuantity<f64>);

#[typetag::serde]
impl QuantityFunction for Constant {
    fn eval(&self, _: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return DynQuantity::from(self.0);
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

#[test]
fn test_functions() {
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Volume> = fun.clone() * constant * fun;
        assert_eq!(prod.eval(&[]).get::<cubic_meter>(), 8.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Area> = constant * fun;
        assert_eq!(prod.eval(&[]).get::<square_meter>(), 4.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Ratio> = fun / constant;
        assert_eq!(prod.eval(&[]).get::<ratio>(), 1.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(4.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<ReciprocalLength> = (fun.clone() + constant.clone()) / constant / fun;
        assert_eq!(prod.eval(&[]).get::<reciprocal_meter>(), 0.75);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Ratio> = constant / fun;
        assert_eq!(prod.eval(&[]).get::<ratio>(), 1.0);
    }
}

#[test]
fn test_clamp() {
    let fun: VarQuantity<Length> =
        VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
            .expect("valid length");
    assert_eq!(fun.eval(&[]).get::<meter>(), 2.0);

    let fun = fun.clamp(Length::new::<meter>(1.0), Length::new::<meter>(1.0));
    assert_eq!(fun.eval(&[]).get::<meter>(), 1.0);

    let sum = VarQuantity::from(Length::new::<meter>(4.0)) + fun;
    assert_eq!(sum.eval(&[]).get::<meter>(), 5.0);
}

#[test]
fn test_sum() {
    let quantites = vec![
        VarQuantity::from(Length::new::<meter>(1.0)),
        VarQuantity::from(Length::new::<meter>(2.0)),
        VarQuantity::from(Length::new::<meter>(3.0)),
    ];
    let sum: VarQuantity<Length> = quantites.into_iter().sum();
    assert_eq!(sum.eval(&[]).get::<meter>(), 6.0);
}

#[test]
fn test_references_with_dedup() {
    {
        let first = VarQuantity::from(Length::new::<meter>(1.0));
        let second = VarQuantity::from(Length::new::<meter>(2.0));
        let sum: VarQuantity<Length> = first + &second + &second;
        assert_eq!(sum.eval(&[]).get::<meter>(), 5.0);
    }
    {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // This function does not want to get cloned and only evaluated twice
        #[derive(Serialize, Deserialize, Default)]
        struct LazyDontClone {
            eval_counter: AtomicUsize,
        }

        impl Clone for LazyDontClone {
            fn clone(&self) -> Self {
                panic!("I really don't want to be cloned!")
            }
        }

        impl PartialEq for LazyDontClone {
            fn eq(&self, other: &Self) -> bool {
                self.eval_counter.load(Ordering::Relaxed)
                    == other.eval_counter.load(Ordering::Relaxed)
            }
        }

        #[typetag::serde]
        impl QuantityFunction for LazyDontClone {
            fn eval(&self, _: &[DynQuantity<f64>]) -> DynQuantity<f64> {
                // Function is evaluated once when building the VarQuantity
                let loaded = self.eval_counter.load(Ordering::Relaxed);
                if loaded > 1 {
                    panic!("Function has been evaluated already!")
                }
                self.eval_counter.store(loaded + 1, Ordering::Relaxed);
                return DynQuantity::new(2.0, PredefUnit::Length);
            }

            fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
                (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
            }
        }

        let leave_me_alone: VarQuantity<Length> =
            VarQuantity::new(LazyDontClone::default()).unwrap();

        // Addition
        let seed: VarQuantity<Length> = VarQuantity::new(LazyDontClone::default()).unwrap();
        let expr: VarQuantity<Length> = seed + &leave_me_alone + &leave_me_alone;
        assert_eq!(expr.eval(&[]).get::<meter>(), 6.0);

        // Subtraction
        let seed: VarQuantity<Length> = VarQuantity::new(LazyDontClone::default()).unwrap();
        let expr: VarQuantity<Length> = seed - &leave_me_alone - &leave_me_alone;
        assert_eq!(expr.eval(&[]).get::<meter>(), -2.0);

        // Multiplication
        let seed: VarQuantity<Length> = VarQuantity::new(LazyDontClone::default()).unwrap();
        let expr: VarQuantity<Area> = seed * &leave_me_alone;
        assert_eq!(expr.eval(&[]).get::<square_meter>(), 4.0);

        // Division
        let seed: VarQuantity<Length> = VarQuantity::new(LazyDontClone::default()).unwrap();
        let expr: VarQuantity<Ratio> = seed / &leave_me_alone;
        assert_eq!(expr.eval(&[]).get::<ratio>(), 1.0);
    }
}

#[test]
fn test_dedup_composed() {
    // Apply the same function multiple times and dedup it in the tokens
    let fun: VarQuantity<Length> =
        VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
            .expect("valid length");

    let fun2: VarQuantity<Length> =
        VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
            .expect("valid length");

    let fun3: VarQuantity<Length> =
        VarQuantity::new(Constant(DynQuantity::new(1.0, PredefUnit::Length)))
            .expect("valid length");

    let expr = fun.clone() + fun.clone() + fun.clone() - fun.clone() / fun2 * fun3;
    assert_eq!(expr.eval(&[]).get::<meter>(), 5.0);

    // Inspect the tokens
    let tokens = expr.tokens();
    assert_matches!(tokens[0], RpnToken::OperandFn(_));
    assert_matches!(tokens[1], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[2], RpnToken::Add);
    assert_matches!(tokens[3], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[4], RpnToken::Add);
    assert_matches!(tokens[5], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[6], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[7], RpnToken::Div);
    assert_matches!(tokens[8], RpnToken::OperandFn(_));
    assert_matches!(tokens[9], RpnToken::Mul);
    assert_matches!(tokens[10], RpnToken::Sub);
}

#[test]
fn test_only_evaluated_once() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    // This test checks if the function is really only evaluated once
    #[derive(Serialize, Deserialize, Default)]
    struct LengthOnlyOnce {
        eval_counter: AtomicUsize,
    }

    impl Clone for LengthOnlyOnce {
        fn clone(&self) -> Self {
            Self {
                eval_counter: AtomicUsize::new(self.eval_counter.load(Ordering::Relaxed)),
            }
        }
    }

    impl PartialEq for LengthOnlyOnce {
        fn eq(&self, other: &Self) -> bool {
            self.eval_counter.load(Ordering::Relaxed) == other.eval_counter.load(Ordering::Relaxed)
        }
    }

    #[typetag::serde]
    impl QuantityFunction for LengthOnlyOnce {
        fn eval(&self, _: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            // Function is evaluated once when building the VarQuantity
            let loaded = self.eval_counter.load(Ordering::Relaxed);
            if loaded > 1 {
                panic!("Function has been evaluated already!")
            }
            self.eval_counter.store(loaded + 1, Ordering::Relaxed);
            return DynQuantity::new(3.0, PredefUnit::Length);
        }

        fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
            (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
        }
    }

    let fun: VarQuantity<Length> =
        VarQuantity::new(LengthOnlyOnce::default()).expect("valid length");

    let expr = VarQuantity::from(Length::new::<meter>(10.0))
        + fun.clone()
        + (fun.clone() + fun.clone()) * fun.clone() / fun.clone()
        - fun;
    assert_eq!(expr.eval(&[]).get::<meter>(), 16.0);
}

#[test]
fn test_dedup_from_tokens() {
    let fun = Constant(DynQuantity::new(2.0, PredefUnit::Length));

    // Build our token chain (which forms a sum)
    let mut tokens = Vec::new();
    tokens.push(RpnToken::OperandConst(DynQuantity::new(
        10.0,
        PredefUnit::Length,
    )));
    tokens.push(RpnToken::OperandFn(Box::new(fun.clone())));
    tokens.push(RpnToken::Add);
    tokens.push(RpnToken::OperandFn(Box::new(fun.clone())));
    tokens.push(RpnToken::Add);
    tokens.push(RpnToken::OperandFn(Box::new(fun.clone())));
    tokens.push(RpnToken::Sub);
    tokens.push(RpnToken::OperandFn(Box::new(fun.clone())));
    tokens.push(RpnToken::Add);

    let composed: VarQuantity<Length> =
        VarQuantity::from_tokens(tokens).expect("valid token chain");
    assert_eq!(composed.eval(&[]).get::<meter>(), 14.0);

    // Inspect the tokens
    let tokens = composed.tokens();
    assert_matches!(tokens[0], RpnToken::OperandConst(_));
    assert_matches!(tokens[1], RpnToken::OperandFn(_));
    assert_matches!(tokens[2], RpnToken::Add);
    assert_matches!(tokens[3], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[4], RpnToken::Add);
    assert_matches!(tokens[5], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[6], RpnToken::Sub);
    assert_matches!(tokens[7], RpnToken::OperandFnRef(0));
    assert_matches!(tokens[8], RpnToken::Add);
}

fn length_fn_token(value: f64) -> RpnToken {
    RpnToken::OperandFn(Box::new(Constant(DynQuantity::new(
        value,
        PredefUnit::Length,
    ))))
}

fn length_const_token(value: f64) -> RpnToken {
    RpnToken::OperandConst(DynQuantity::new(value, PredefUnit::Length))
}

#[test]
fn test_invalid_unary_operator() {
    let tokens = vec![RpnToken::Power(2)];

    let result = VarQuantity::<Length>::from_tokens(tokens);

    assert!(matches!(
        result,
        Err(EvalError::MissingOperand {
            operator: "unary",
            required: 1,
            available: 0,
            ..
        })
    ));
}

#[test]
fn test_invalid_binary_operator() {
    let tokens = vec![length_const_token(1.0), RpnToken::Add];

    let result = VarQuantity::<Length>::from_tokens(tokens);

    assert!(matches!(
        result,
        Err(EvalError::MissingOperand {
            operator: "binary",
            required: 2,
            available: 1,
            ..
        })
    ));
}

#[test]
fn test_invalid_function_reference() {
    let tokens = vec![
        length_fn_token(1.0),
        RpnToken::OperandFnRef(1),
        RpnToken::Add,
    ];

    let result = VarQuantity::<Length>::from_tokens(tokens);
    assert!(matches!(
        result,
        Err(EvalError::InvalidFunctionReference {
            function_index: 1,
            ..
        })
    ));
}

#[test]
fn test_forward_function_reference() {
    let tokens = vec![RpnToken::OperandFnRef(0), length_fn_token(1.0)];

    let result = VarQuantity::<Length>::from_tokens(tokens);

    assert!(matches!(
        result,
        Err(EvalError::InvalidFunctionReference {
            token_index: 0,
            function_index: 0,
        })
    ));
}

#[test]
fn test_invalid_stack_depth() {
    let tokens = vec![length_const_token(1.0), length_const_token(2.0)];

    let result = VarQuantity::<Length>::from_tokens(tokens);

    assert!(matches!(
        result,
        Err(EvalError::InvalidStackDepth { depth: 2 })
    ));
}

#[test]
fn test_empty_expression() {
    let tokens = Vec::new();

    let result = VarQuantity::<Length>::from_tokens(tokens);

    assert!(matches!(
        result,
        Err(EvalError::InvalidStackDepth { depth: 0 })
    ));
}
