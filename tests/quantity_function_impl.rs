use serde::{Deserialize, Serialize};
use var_quantity::uom::si::{
    electric_current::ampere, electric_potential::volt, f64::*, frequency::hertz,
    magnetic_flux_density::tesla, power::watt, thermodynamic_temperature::degree_celsius,
};
use var_quantity::{DynQuantity, PredefUnit, Unit};
use var_quantity::{QuantityFunction, VarQuantity};

#[test]
fn test_var_quantity() {
    let conditions = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<degree_celsius>(2.0).into(),
    ];

    let var_quantity = VarQuantity::from(Power::new::<watt>(1.0));
    assert_eq!(var_quantity.eval(conditions.as_slice()).get::<watt>(), 1.0);
}

#[test]
fn test_multiply_by_current() {
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    struct MultiplyIfCurrent(ElectricPotential);

    let conditions = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<degree_celsius>(2.0).into(),
    ];

    #[typetag::serde]
    impl QuantityFunction for MultiplyIfCurrent {
        fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let value = conditions
                .into_iter()
                .find_map(|iq| {
                    if Unit::from(PredefUnit::ElectricCurrent) == iq.unit {
                        return Some(iq.clone());
                    } else {
                        return None;
                    }
                })
                .unwrap_or(DynQuantity::new(0.0, PredefUnit::ElectricCurrent));
            return DynQuantity::from(self.0) * value;
        }

        fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
            (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
        }
    }

    let wrapper = Box::new(MultiplyIfCurrent(ElectricPotential::new::<volt>(0.5)));

    // Test deref
    fn do_nothing_ref(_function: &dyn QuantityFunction) {}
    do_nothing_ref(&*wrapper);
    fn do_nothing_any(_function: &dyn std::any::Any) {}
    do_nothing_any(&*wrapper);

    // Test casting
    assert!(
        (wrapper.as_ref() as &dyn std::any::Any)
            .downcast_ref::<MultiplyIfCurrent>()
            .is_some()
    );

    let var_quantity: VarQuantity<Power> =
        VarQuantity::new(MultiplyIfCurrent(ElectricPotential::new::<volt>(0.5)))
            .expect("evaluates to power");
    assert_eq!(var_quantity.eval(conditions.as_slice()).get::<watt>(), 1.0);
}

#[test]
fn test_readme_example() {
    // Model 1: p = k * B^2
    #[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
    struct Model1(DynQuantity<f64>);

    #[typetag::serde]
    impl QuantityFunction for Model1 {
        fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
            for factor in conditions.iter() {
                if b.unit == factor.unit {
                    b = factor.clone();
                }
            }
            return self.0 * b.powi(2);
        }

        fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
            (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
        }
    }

    // Model 2: p = k * f^2 * B^2
    #[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
    struct Model2(DynQuantity<f64>);

    #[typetag::serde]
    impl QuantityFunction for Model2 {
        fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
            let mut f = DynQuantity::new(0.0, PredefUnit::Frequency);
            for factor in conditions.iter() {
                if b.unit == factor.unit {
                    b = factor.clone();
                }
                if f.unit == factor.unit {
                    f = factor.clone();
                }
            }
            return self.0 * f.powi(2) * b.powi(2);
        }

        fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
            (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
        }
    }

    let k = DynQuantity::new(
        1000.0,
        Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2),
    );
    let model1: VarQuantity<Power> = VarQuantity::new(Model1(k)).expect("output unit is watt");

    let k = DynQuantity::new(
        2.0,
        Unit::from(PredefUnit::Power)
            / Unit::from(PredefUnit::MagneticFluxDensity).powi(2)
            / Unit::from(PredefUnit::Frequency).powi(2),
    );
    let model2: VarQuantity<Power> = VarQuantity::new(Model2(k)).expect("output unit is watt");

    // This function takes a variable quantity, the magnetic flux density and
    // the frequency and calculates the losses
    fn losses(model: &VarQuantity<Power>, b: MagneticFluxDensity, f: Frequency) -> Power {
        return model.eval(&[b.into(), f.into()]);
    }

    let b = MagneticFluxDensity::new::<tesla>(1.2);
    let f = Frequency::new::<hertz>(20.0);

    assert_eq!(losses(&model1, b, f).get::<watt>(), 1440.0);
    assert_eq!(losses(&model2, b, f).get::<watt>(), 1152.0);
}
