var_quantity
============

<!-- This file has ben generated with build.rs by concatenating docs/links.md,
docs/main.md and (if available docs/end.md). Do not modify this file, instead
modify the components. -->

[`VarQuantity`]: https://docs.rs/var_quantity/0.6.0/var_quantity/struct.VarQuantity.html
[`VarQuantity::eval`]: https://docs.rs/var_quantity/0.6.0/var_quantity/struct.VarQuantity.html#method.get
[`VarQuantity::new`]: https://docs.rs/var_quantity/0.6.0/var_quantity/struct.VarQuantity.html#method.new
[`QuantityFunction`]: https://docs.rs/var_quantity/0.6.0/var_quantity/trait.QuantityFunction.html
[`QuantityFunction::eval`]: https://docs.rs/var_quantity/0.6.0/var_quantity/trait.QuantityFunction.html#tymethod.call
[`ComposedQuantityFunction`]: https://docs.rs/var_quantity/0.6.0/var_quantity/struct.ComposedQuantityFunction.html
[`filter_unary_function`]: https://docs.rs/var_quantity/0.6.0/var_quantity/fn.filter_unary_function.html
[`FirstOrderTaylor`]: https://docs.rs/var_quantity/0.6.0/var_quantity/unary/struct.FirstOrderTaylor.html
[`Polynomial`]: https://docs.rs/var_quantity/0.6.0/var_quantity/unary/struct.Polynomial.html
[`unary`]: https://docs.rs/var_quantity/0.6.0/var_quantity/unary.html

[![Documentation](https://docs.rs/var_quantity/badge.svg)](https://docs.rs/var_quantity)

This crate provides a composable interface for defining and evaluating variable
physical quantities whose values depend on other quantities.

The full API documentation is available at <https://docs.rs/var_quantity/0.6.0/var_quantity>.

> **Feedback welcome!**  
> Found a bug, missing docs, or have a feature request?  
> Please open an issue on [GitHub](https://github.com/StefanMathis/var_quantity).

In physical equations, quantities often depend on environmental conditions.
For example, when calculating the current through two resistors connected in
series, one has to account for their temperature dependencies. Consequently,
the resulting current is also a function of temperature:

`i(T) = u / (r1(T) + r2(T))`

The following example shows how to express and compose this model using the
[`VarQuantity`] type provided by this crate:

```rust
use approxim::assert_abs_diff_eq;

use var_quantity::unary::FirstOrderTaylor;
use var_quantity::uom::si::f64::*;
use var_quantity::uom::si::{
    electric_current::ampere, electric_potential::volt, electrical_resistance::ohm,
    temperature_interval::kelvin, thermodynamic_temperature::degree_celsius,
};
use var_quantity::*;

// Model the electrical resistance as a first-order Taylor approximation:
// r = r0 * (1 + k * (t - t0))
// at the reference temperature:
let t0 = ThermodynamicTemperature::new::<degree_celsius>(20.0);

// First resistor: Resistance increases by 50 % per 100 K
let r0_1 = ElectricalResistance::new::<ohm>(2.0);
let k_1 = 0.005 / TemperatureInterval::new::<kelvin>(1.0);
let r1: VarQuantity<ElectricalResistance> = VarQuantity::new(
    FirstOrderTaylor::new(r0_1.into(), k_1.into(), t0.into()).expect("k * t0 is dimensionless"),
)
.expect("model output must have units of resistance");

// Second resistor: Resistance increases by 20 % per 100 K
let r0_2 = ElectricalResistance::new::<ohm>(1.0);
let k_2 = 0.002 / TemperatureInterval::new::<kelvin>(1.0);
let r2: VarQuantity<ElectricalResistance> = VarQuantity::new(
    FirstOrderTaylor::new(r0_2.into(), k_2.into(), t0.into()).expect("k * t0 is dimensionless"),
)
.expect("model output must have units of resistance");

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
```

As demonstrated by the example, this crate builds on
[`uom`](https://crates.io/crates/uom) and
[`dyn_quantity`](https://crates.io/crates/dyn_quantity).
The former ensures compile-time dimensional type safety, while the latter
provides the underlying runtime representation of physical quantities without
introducing additional generics into the model implementations.

# Defining models for physical quantities

The previous example uses the predefined [`FirstOrderTaylor`] model to
represent temperature-dependent resistance. By implementing the
[`QuantityFunction`] trait, you can define arbitrary models yourself:

```rust
use var_quantity::*;
use var_quantity::uom::si::{f64::{Power, MagneticFluxDensity, Frequency}, 
    power::watt, magnetic_flux_density::tesla, frequency::hertz};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.

// Eddy current loss model: p = k * f^2 * B^2
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
struct EddyCurrentLosses(DynQuantity<f64>);

// Implement QuantityFunction to make EddyCurrentLosses usable as a
// non-constant VarQuantity.
#[typetag::serde]
impl QuantityFunction for EddyCurrentLosses {
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        // For this model, missing flux density or frequency is interpreted as zero.
        // Consequently, the calculated losses are also zero.
        let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
        let mut f = DynQuantity::new(0.0, PredefUnit::Frequency);

        for condition in conditions {
            if b.unit == condition.unit {
                b = *condition;
            }
            if f.unit == condition.unit {
                f = *condition;
            }
        }

        self.0 * f.powi(2) * b.powi(2)
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

// Define the proportionality constant with unit W / (T² * Hz²).
let k = DynQuantity::new(
    2.0,
    Unit::from(PredefUnit::Power)
        / Unit::from(PredefUnit::MagneticFluxDensity).powi(2)
        / Unit::from(PredefUnit::Frequency).powi(2),
);

// VarQuantity::new checks at runtime if the output unit of
// EddyCurrentLosses::eval matches the unit of Power
let model: VarQuantity<Power> = VarQuantity::new(EddyCurrentLosses(k)).expect("output unit is watt");

// Provide the known environmental conditions: A sinusoidal magnetic field
// with an amplitude of 1.2 T and a frequency of 20 Hz.
let b = MagneticFluxDensity::new::<tesla>(1.2);
let f = Frequency::new::<hertz>(20.0);

// DynQuantity allows us to group b and f together by representing the underlying units at runtime.
let conditions: &[DynQuantity<f64>] = &[b.into(), f.into()];

// Calculate the losses for the specified conditions.
assert_eq!(model.eval(conditions).get::<watt>(), 1152.0);

// Without a magnetic field or frequency, the model assumes zero losses.
assert_eq!(model.eval(&[]).get::<watt>(), 0.0);
```

A [`VarQuantity`] can be evaluated with a collection of environmental
conditions. These conditions are not necessarily all used by the underlying
model: each [`QuantityFunction`] implementation is responsible for extracting
the quantities relevant to its physical relationship. Missing conditions can be
interpreted according to the assumptions of the respective model. For example,
the loss model from the previous example assumes a zero magnetic field if no
magnetic field is supplied. 

This allows different models to be composed without requiring the caller
to know their individual dependencies. The caller simply supplies the
environmental conditions available to them, while each model handles
the conditions relevant to its own implementation.

# Predefined variable quantity models

Some variable quantity models are common enough to be provided by this crate.
For example, if we only needed an eddy-current loss model for a fixed
frequency, we could use the [`Polynomial`] struct from the [`unary`] module
instead of defining a custom model:

```rust
use var_quantity::*;
use var_quantity::unary::Polynomial;
use var_quantity::uom::si::{f64::{Power, MagneticFluxDensity}, 
    power::watt, magnetic_flux_density::tesla};

// The coefficients [a, b, c] define the polynomial ax² + bx + c.
// Their units must be consistent with the input and output units:
// [a] = W/T², [b] = W/T, [c] = W.
//
// The output unit is determined by [c], and the input unit by [c/b].
// Polynomial::new checks that all coefficients have consistent units.
let a = DynQuantity::new(800.0, Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2));
let b = DynQuantity::new(0.0, Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity));
let c = DynQuantity::new(0.0, PredefUnit::Power);
let polynomial = Polynomial::new(vec![a, b, c]).expect("output units of terms match");

// VarQuantity::new checks at runtime if the output unit of
// Polynomial::eval matches the unit of Power
let model: VarQuantity<Power> = VarQuantity::new(polynomial).expect("output unit is watt");

// Calculate the losses for a field with 1.2 T.
assert_eq!(model.eval(&[MagneticFluxDensity::new::<tesla>(1.2).into()]).get::<watt>(), 1152.0);
```

For a full list of available models, see the following modules:
- [`unary`] : Models representing unary functions (single input). 

# Serialization and deserialization

The serde integration is gated behind the `serde` feature flag.

All public structs and enums in this crate implement serialization and
deserialization. See the docstrings of the individual types for details.

The `from_str` feature extends the `serde` feature by enabling deserialization
of a constant [`VarQuantity`] directly from a string via the
[`dyn_quantity`](https://crates.io/crates/dyn_quantity) crate. See
[dyn_quantity::from_str_impl](https://docs.rs/dyn_quantity/latest/dyn_quantity/quantity/from_str_impl/index.html)
for details.

```rust
use var_quantity::VarQuantity;
use var_quantity::uom::si::electrical_resistivity::ohm_meter;
use var_quantity::uom::si::f64::ElectricalResistivity;
use yaml_serde;

let q: VarQuantity<ElectricalResistivity> =
    yaml_serde::from_str("1/(2.0e6) Ohm*m").unwrap();
assert_eq!(q.eval(&[]).get::<ohm_meter>(), 0.5e-6)
```