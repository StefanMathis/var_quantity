use serde::{Deserialize, Serialize};
use var_quantity::uom::si::f64::*;
use var_quantity::uom::si::{area::square_meter, length::meter, ratio::ratio};
use var_quantity::*;

#[test]
fn test_two_constants() {
    {
        let lhs = VarQuantity::Constant(Length::new::<meter>(2.0));
        let rhs = VarQuantity::Constant(Length::new::<meter>(2.0));

        let prod: VarQuantity<Area> = lhs * rhs;
        assert_eq!(prod.get(&[]).get::<square_meter>(), 4.0);
    }
    {
        let lhs = VarQuantity::Constant(Length::new::<meter>(2.0));
        let rhs = VarQuantity::Constant(Length::new::<meter>(2.0));

        let quot: VarQuantity<Ratio> = lhs / rhs;
        assert_eq!(quot.get(&[]).get::<ratio>(), 1.0);
    }
    {
        let lhs = VarQuantity::Constant(Area::new::<square_meter>(2.0));
        let rhs = VarQuantity::Constant(Length::new::<meter>(2.0));

        let quot: VarQuantity<Length> = lhs / rhs;
        assert_eq!(quot.get(&[]).get::<meter>(), 1.0);
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

    fn mul(&mut self, value: DynQuantity<f64>) -> bool {
        self.0 = self.0 * value;
        true
    }
}

#[test]
fn test_mul() {
    let mut fun = Constant(DynQuantity::new(2.0, PredefUnit::Length));
    assert_eq!(fun.eval(&[]).value, 2.0);

    assert!(fun.mul(DynQuantity::new(2.0, PredefUnit::Length)));
    assert_eq!(fun.eval(&[]).value, 4.0);
}

#[test]
fn test_constant_and_function() {
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Area> = fun * constant;
        assert_eq!(prod.get(&[]).get::<square_meter>(), 4.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Area> = constant * fun;
        assert_eq!(prod.get(&[]).get::<square_meter>(), 4.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Ratio> = fun / constant;
        assert_eq!(prod.get(&[]).get::<ratio>(), 1.0);
    }
    {
        let fun: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let constant = VarQuantity::from(Length::new::<meter>(2.0));

        let prod: VarQuantity<Ratio> = constant / fun;
        assert_eq!(prod.get(&[]).get::<ratio>(), 1.0);
    }
}

#[test]
fn test_two_functions() {
    {
        let fun1: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let fun2: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");

        let prod: VarQuantity<Area> = fun1 * fun2;
        assert_eq!(prod.get(&[]).get::<square_meter>(), 4.0);
    }
    {
        let fun1: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");
        let fun2: VarQuantity<Length> =
            VarQuantity::new(Constant(DynQuantity::new(2.0, PredefUnit::Length)))
                .expect("valid length");

        let quot: VarQuantity<Ratio> = fun1 / fun2;
        assert_eq!(quot.get(&[]).get::<ratio>(), 1.0);
    }
}
