use std::assert_matches;

use dyn_quantity::uom::si::{
    electric_current::ampere, electrical_resistance::ohm, electrical_resistivity::ohm_meter,
    f64::*, magnetic_flux_density::tesla, power::watt, thermodynamic_temperature::kelvin,
};
use dyn_quantity::{DynQuantity, PredefUnit, Unit, deserialize_quantity};
use indoc::indoc;
use serde::{Deserialize, Serialize};
use var_quantity::{unary::FirstOrderTaylor, *};

#[test]
fn test_deserialize_constant_success() {
    {
        // No units
        let q: VarQuantity<MagneticFluxDensity> = serde_yaml::from_str("0.001").unwrap();
        assert_eq!(q.eval(&[]).get::<tesla>(), 0.001)
    }
    {
        // No units
        let q: VarQuantity<MagneticFluxDensity> = serde_yaml::from_str("1000").unwrap();
        assert_eq!(q.eval(&[]).get::<tesla>(), 1000.0)
    }
    {
        // No units
        let q: VarQuantity<MagneticFluxDensity> = serde_yaml::from_str("-1000").unwrap();
        assert_eq!(q.eval(&[]).get::<tesla>(), -1000.0)
    }
    {
        // With units
        let q: VarQuantity<MagneticFluxDensity> = serde_yaml::from_str("1 mT").unwrap();
        assert_eq!(q.eval(&[]).get::<tesla>(), 0.001)
    }
    {
        // With units
        let q: VarQuantity<MagneticFluxDensity> = serde_yaml::from_str("1e-3 T").unwrap();
        assert_eq!(q.eval(&[]).get::<tesla>(), 0.001)
    }
    {
        // With units
        let q: VarQuantity<ElectricalResistivity> =
            serde_yaml::from_str("1/(2.0e6) Ohm*m").unwrap();
        assert_eq!(q.eval(&[]).get::<ohm_meter>(), 0.5e-6)
    }
}

#[test]
fn test_deserialize_constant_fail() {
    {
        // Unit mismatch
        assert!(serde_yaml::from_str::<VarQuantity<Power>>("1 mT").is_err());
    }
}

#[test]
fn test_rpn_token_serde() {
    let yaml = r#"
OperandConst:
  value: 1.0
  unit:
    second: -2
    meter: 0
    kilogram: 1
    ampere: -1
    kelvin: 0
    mol: 0
    candela: 0
"#;

    let token: RpnToken = serde_yaml::from_str(yaml).expect("deserializable");

    assert_matches!(token, RpnToken::OperandConst(_));
}

#[test]
fn test_deserialize_tokens() {
    let yaml = r#"
tokens:
- OperandConst:
    value: 1.0
    unit:
        second: 0
        meter: 1
        kilogram: 0
        ampere: 0
        kelvin: 0
        mol: 0
        candela: 0
"#;

    assert!(serde_yaml::from_str::<VarQuantity<Length>>(yaml).is_ok());
    assert!(serde_yaml::from_str::<VarQuantity<Mass>>(yaml).is_err());
}

#[test]
fn test_serialize_and_deserialize() {
    {
        // Constant
        let q = VarQuantity::from(MagneticFluxDensity::new::<tesla>(1.0));
        let string = serde_yaml::to_string(&q).expect("serializable");
        let q_serde: VarQuantity<MagneticFluxDensity> =
            serde_yaml::from_str(&string).expect("deserializable");
        assert_eq!(q_serde.eval(&[]).get::<tesla>(), 1.0);
    }
    {
        // Function
        let fun = FirstOrderTaylor::new(
            DynQuantity::new(2.5, PredefUnit::Power),
            DynQuantity::new(2.0, Unit::from(PredefUnit::ElectricCurrent).powi(-1)),
            DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
        )
        .unwrap();
        let q: VarQuantity<Power> = VarQuantity::new(fun).unwrap();
        let string = serde_yaml::to_string(&q).expect("serializable");
        let q_serde: VarQuantity<Power> = serde_yaml::from_str(&string).expect("deserializable");
        assert_eq!(q_serde.eval(&[]).get::<watt>(), 2.5);
    }
}

// A simple function for a variable electric resistance. If one of the
// conditions is a temperature, divide it by 10 K and add it to the
// base value, otherwise just return the base value
#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct VariableResistance {
    #[serde(deserialize_with = "deserialize_quantity")]
    base_value: ElectricalResistance,
}

#[typetag::serde]
impl QuantityFunction for VariableResistance {
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let quantity = conditions
            .into_iter()
            .find_map(|iq| {
                if Unit::from(PredefUnit::Temperature) == iq.unit {
                    return Some(self.base_value * iq.value / 10.0);
                } else {
                    return None;
                }
            })
            .unwrap_or(self.base_value);
        return quantity.into();
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

#[test]
fn test_deserialize_var_resistance() {
    let conditions = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<kelvin>(20.0).into(),
    ];

    {
        // No units
        let string = indoc! {"
        ---
        VariableResistance:
            base_value: 2.0
        "};

        let var_quantity: VarQuantity<ElectricalResistance> = serde_yaml::from_str(string).unwrap();

        assert_eq!(var_quantity.eval(conditions.as_slice()).get::<ohm>(), 4.0);
    }
    {
        // With units
        let string = indoc! {"
        ---
        VariableResistance:
            base_value: 2.0 mOhm
        "};

        let var_quantity: VarQuantity<ElectricalResistance> = serde_yaml::from_str(string).unwrap();

        assert_eq!(var_quantity.eval(conditions.as_slice()).get::<ohm>(), 0.004);
    }
}

#[test]
fn test_deserialize_clamped_var_resistance() {
    let string = indoc! {"
        tokens:
            - OperandFn:
                VariableResistance:
                  base_value: 2.0 mOhm
            - Clamp:
                min: 1.0
                max: 2.0
        "};
    let var_quantity: VarQuantity<ElectricalResistance> = serde_yaml::from_str(string).unwrap();
    assert_eq!(var_quantity.eval(&[]).get::<ohm>(), 1.0);
}

#[test]
fn test_serialize_and_deserialize_var_resistance() {
    let conditions = [
        ElectricCurrent::new::<ampere>(2.0).into(),
        ThermodynamicTemperature::new::<kelvin>(20.0).into(),
    ];
    {
        let multiply_by_volt = VariableResistance {
            base_value: ElectricalResistance::new::<ohm>(2.0),
        };
        let boxed: Box<dyn QuantityFunction> = Box::new(multiply_by_volt);
        let string = serde_yaml::to_string(&boxed).unwrap();
        let var_quantity: VarQuantity<ElectricalResistance> =
            serde_yaml::from_str(&string).unwrap();

        assert_eq!(var_quantity.eval(conditions.as_slice()).get::<ohm>(), 4.0);
    }
}
