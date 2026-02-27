use dyn_quantity::uom::si::{f64::Length, length::meter};
use var_quantity::{VarQuantity, unary::*};

#[test]
fn test_implementor_eq() {
    let term0 = ExpTerm {
        amplitude: 2.0.into(),
        exponent: 2.0.into(),
    };
    let term1 = ExpTerm {
        amplitude: (-3.0).into(),
        exponent: 0.0.into(),
    };
    let term2 = ExpTerm {
        amplitude: (-2.0).into(),
        exponent: 0.0.into(),
    };

    let fun1: VarQuantity<f64> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term1.clone()]).unwrap(),
    )
    .unwrap();
    let fun2: VarQuantity<f64> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term1.clone()]).unwrap(),
    )
    .unwrap();
    let fun3: VarQuantity<f64> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term2.clone()]).unwrap(),
    )
    .unwrap();

    assert_eq!(fun1, fun2);
    assert_ne!(fun1, fun3);
    assert_ne!(fun2, fun3);
}

#[test]
fn test_implementor_eq_units() {
    let term0: ExpTerm = ExpTerm {
        amplitude: Length::new::<meter>(2.0).into(),
        exponent: 2.0.into(),
    };
    let term1 = ExpTerm {
        amplitude: Length::new::<meter>(-3.0).into(),
        exponent: 0.0.into(),
    };
    let term2 = ExpTerm {
        amplitude: Length::new::<meter>(-2.0).into(),
        exponent: 0.0.into(),
    };

    let fun1: VarQuantity<Length> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term1.clone()]).unwrap(),
    )
    .unwrap();
    let fun2: VarQuantity<Length> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term1.clone()]).unwrap(),
    )
    .unwrap();
    let fun3: VarQuantity<Length> = VarQuantity::try_from_quantity_function(
        Exponential::new(vec![term0.clone(), term2.clone()]).unwrap(),
    )
    .unwrap();

    assert_eq!(fun1, fun2);
    assert_ne!(fun1, fun3);
    assert_ne!(fun2, fun3);
}
