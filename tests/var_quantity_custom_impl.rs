use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use serde::{Deserialize, Serialize};
use uom::si::{
    electric_current::ampere, electric_potential::volt, f64::*, power::watt,
    thermodynamic_temperature::degree_celsius,
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
