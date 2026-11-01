use approxim::assert_abs_diff_eq;

use var_quantity::unary::FirstOrderTaylor;
use var_quantity::uom::si::f64::*;
use var_quantity::uom::si::{
    electric_current::ampere, electric_potential::volt, electrical_resistance::ohm,
    temperature_interval::kelvin, thermodynamic_temperature::degree_celsius,
};
use var_quantity::*;

fn main() {
    // Model the electrical resistance as a first-order Taylor approximation:
    // r = r0 * (1 + k * (t - t0))
    // with the reference temperature:
    let t0 = ThermodynamicTemperature::new::<degree_celsius>(20.0);

    // First resistor: Resistance increases by 50 % per 100 K
    let r0_1 = ElectricalResistance::new::<ohm>(2.0);
    let k_1 = 0.005 / TemperatureInterval::new::<kelvin>(1.0);
    let r1: VarQuantity<ElectricalResistance> = VarQuantity::new(
        FirstOrderTaylor::new(r0_1.into(), k_1.into(), t0.into()).expect("k * t0 is dimensionless"),
    )
    .expect("output can be converted into a resistance");

    // Second resistor: Resistance increases by 20 % per 100 K
    let r0_2 = ElectricalResistance::new::<ohm>(1.0);
    let k_2 = 0.002 / TemperatureInterval::new::<kelvin>(1.0);
    let r2: VarQuantity<ElectricalResistance> = VarQuantity::new(
        FirstOrderTaylor::new(r0_2.into(), k_2.into(), t0.into()).expect("k * t0 is dimensionless"),
    )
    .expect("output can be converted into a resistance");

    // The voltage is constant, but must also be represented as a VarQuantity
    // to participate in the expression.
    let u: VarQuantity<ElectricPotential> = VarQuantity::from(ElectricPotential::new::<volt>(10.0));

    // Combine the two independently defined resistance models in series.
    // The resulting expression retains the temperature dependencies of both.
    let r_total = r1 + r2;

    // Build the current equation i(T) = u / r_total.
    // The dimensional equation ElectricCurrent = ElectricPotential / ElectricalResistance
    // is checked at compile time.
    let i: VarQuantity<ElectricCurrent> = u / r_total;
    // let l: VarQuantity<Length> = u / (r1 + r2); // This does not compile due to the dimensional analysis

    // Evaluate the composed expression for different ambient temperatures.
    let cold: &[DynQuantity<f64>] = &[ThermodynamicTemperature::new::<degree_celsius>(20.0).into()];
    assert_abs_diff_eq!(i.eval(cold).get::<ampere>(), 3.3333, epsilon = 1e-3);

    let hot: &[DynQuantity<f64>] = &[ThermodynamicTemperature::new::<degree_celsius>(120.0).into()];
    assert_abs_diff_eq!(i.eval(hot).get::<ampere>(), 2.380, epsilon = 1e-3);
}
