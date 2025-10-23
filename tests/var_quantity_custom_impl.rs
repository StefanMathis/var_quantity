use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use serde::{Deserialize, Serialize};
use uom::si::{
    electric_current::ampere, electric_potential::volt, f64::*, frequency::hertz,
    magnetic_flux_density::tesla, power::watt, thermodynamic_temperature::degree_celsius,
};
use var_quantity::{FunctionWrapper, QuantityFunction, VarQuantity};

#[test]
fn test_var_quantity() {
    let influencing_factors = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<degree_celsius>(2.0).into(),
    ];

    let var_quantity = VarQuantity::Constant(Power::new::<watt>(1.0));
    assert_eq!(
        var_quantity
            .get(influencing_factors.as_slice())
            .get::<watt>(),
        1.0
    );
}

#[test]
fn test_multiply_by_current() {
    #[derive(Serialize, Deserialize, Clone)]
    struct MultiplyIfCurrent(ElectricPotential);

    let influencing_factors = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<degree_celsius>(2.0).into(),
    ];

    #[typetag::serde]
    impl QuantityFunction for MultiplyIfCurrent {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let value = influencing_factors
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
    }

    let wrapper = FunctionWrapper::new(Box::new(MultiplyIfCurrent(
        ElectricPotential::new::<volt>(0.5),
    )))
    .unwrap();

    let var_quantity: VarQuantity<Power> = VarQuantity::Function(wrapper);
    assert_eq!(
        var_quantity
            .get(influencing_factors.as_slice())
            .get::<watt>(),
        1.0
    );
}

#[test]
fn test_readme_example() {
    // Model 1: p = k * B^2
    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct Model1(DynQuantity<f64>);

    #[typetag::serde]
    impl QuantityFunction for Model1 {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
            for factor in influencing_factors.iter() {
                if b.unit == factor.unit {
                    b = factor.clone();
                }
            }
            return self.0 * b.powi(2);
        }
    }

    // Model 2: p = k * f^2 * B^2
    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct Model2(DynQuantity<f64>);

    #[typetag::serde]
    impl QuantityFunction for Model2 {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
            let mut f = DynQuantity::new(0.0, PredefUnit::Frequency);
            for factor in influencing_factors.iter() {
                if b.unit == factor.unit {
                    b = factor.clone();
                }
                if f.unit == factor.unit {
                    f = factor.clone();
                }
            }
            return self.0 * f.powi(2) * b.powi(2);
        }
    }

    let k = DynQuantity::new(
        1000.0,
        Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2),
    );
    let model1: VarQuantity<Power> = VarQuantity::Function(
        FunctionWrapper::new(Box::new(Model1(k))).expect("output unit is watt"),
    );

    let k = DynQuantity::new(
        2.0,
        Unit::from(PredefUnit::Power)
            / Unit::from(PredefUnit::MagneticFluxDensity).powi(2)
            / Unit::from(PredefUnit::Frequency).powi(2),
    );
    let model2: VarQuantity<Power> = VarQuantity::Function(
        FunctionWrapper::new(Box::new(Model2(k))).expect("output unit is watt"),
    );

    // This function takes a variable quantity, the magnetic flux density and
    // the frequency and calculates the losses
    fn losses(model: &VarQuantity<Power>, b: MagneticFluxDensity, f: Frequency) -> Power {
        return model.get(&[b.into(), f.into()]);
    }

    let b = MagneticFluxDensity::new::<tesla>(1.2);
    let f = Frequency::new::<hertz>(20.0);

    assert_eq!(losses(&model1, b, f).get::<watt>(), 1440.0);
    assert_eq!(losses(&model2, b, f).get::<watt>(), 1152.0);
}
