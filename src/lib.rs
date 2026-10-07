/*!
[`VarQuantity`]: crate::VarQuantity
[`VarQuantity::Constant`]: crate::VarQuantity::Constant
[`VarQuantity::Function`]: crate::VarQuantity::Function
[`VarQuantity::get`]: crate::VarQuantity::get
[`VarQuantity::new`]: crate::QuantityFunction::new
[`QuantityFunction`]: crate::QuantityFunction
[`QuantityFunction::call`]: crate::QuantityFunction::call
[`filter_unary_function`]: crate::filter_unary_function
[`Polynomial`]: crate::unary::Polynomial
[`unary`]: crate::unary
[typetag]: typetag
[uom]: uom

Composable serialization and deserialization for Rust structs.

 */
#![doc = include_str!("../docs/main.md")]
#![deny(missing_docs)]

pub use dyn_quantity::*;

use num::Complex;

#[cfg(feature = "serde")]
pub use typetag;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use dyn_quantity;
pub mod unary;

/**
This is a marker trait which defines trait bounds for all types `T` which can
be used as "quantities" in [`VarQuantity<T>`]. It does not provide any methods
and is auto-implemented for all `T` fulfilling the bounds, hence it is not
necessary to ever import this trait. It is only public to make compiler error
messages more helpful.
 */
#[cfg(not(feature = "serde"))]
pub trait QuantityF64:
    UnitFromType
    + TryFrom<DynQuantity<Complex<f64>>>
    + Clone
    + std::fmt::Debug
    + Into<DynQuantity<Complex<f64>>>
    + PartialEq
    + Default
{
}

/**
This is a marker trait which defines trait bounds for all types `T` which can
be used as "quantities" in [`VarQuantity<T>`]. It does not provide any methods
and is auto-implemented for all `T` fulfilling the bounds, hence it is not
necessary to ever import this trait. It is only public to make compiler error
messages more helpful.
 */
#[cfg(feature = "serde")]
pub trait QuantityF64:
    UnitFromType
    + TryFrom<DynQuantity<Complex<f64>>>
    + Clone
    + std::fmt::Debug
    + Into<DynQuantity<Complex<f64>>>
    + Serialize
    + PartialEq
    + Default
{
}

#[cfg(feature = "serde")]
impl<T> QuantityF64 for T where
    T: UnitFromType
        + TryFrom<DynQuantity<Complex<f64>>>
        + Clone
        + std::fmt::Debug
        + Into<DynQuantity<Complex<f64>>>
        + serde::Serialize
        + PartialEq
        + Default
{
}

#[cfg(not(feature = "serde"))]
impl<T> QuantityF64 for T where
    T: UnitFromType
        + TryFrom<DynQuantity<Complex<f64>>>
        + Clone
        + std::fmt::Debug
        + Into<DynQuantity<Complex<f64>>>
        + PartialEq
        + Default
{
}

/**
Trait used to construct variable quantities whose value is a (pure) function of
other quantities.

Implementing this trait for a type marks it as being a variable quantity, whose
value can change under the influence of other quantities. For example, a
resistance can be a function of temperature:

```
use var_quantity::{DynQuantity, PredefUnit, Unit, QuantityFunction};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
struct Resistance;

// Again, the macro annotation is just here because of the serde feature
#[typetag::serde]
impl QuantityFunction for Resistance {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut temperature = 0.0;
        let temperature_unit: Unit = PredefUnit::Temperature.into();
        for f in conditions.iter() {
            if f.unit == temperature_unit {
                temperature = f.value;
                break;
            }
        }
        return DynQuantity::new(1.0 + temperature / 100.0, PredefUnit::ElectricResistance);
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

// Influencing factors
let infl1 = &[DynQuantity::new(6.0, PredefUnit::ElectricCurrent)];
let infl2 = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
    DynQuantity::new(20.0, PredefUnit::Temperature),
];

let resistance = Resistance {};

assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricResistance), resistance.call(&[]));
assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricResistance), resistance.call(infl1));
assert_eq!(DynQuantity::new(1.2, PredefUnit::ElectricResistance), resistance.call(infl2));
```

An important constraint which unfortunately cannot be covered by the type
system is that the [`DynQuantity<f64>`] returned by [`QuantityFunction::call`]
must always have the same [`Unit`] field. See the [Features](#features) section
and the docstring of [`VarQuantity`] for details.

# Features

When the `serde` feature is enabled, any type implementing [`QuantityFunction`]
can be serialized / deserialized as a trait object using the
[typetag](https://docs.rs/typetag/latest/typetag/) crate. This has the following
implications:
- [`QuantityFunction::call`] cannot return a generic type (limitation of
typetag), which is why the dynamic [`DynQuantity`] type is used.
- When implementing [`QuantityFunction`] for a type, the `#[typetag::serde]`
annotation must be applied to the `impl` block (see example).

In turn, this feature enables serialization / deserialization of [`VarQuantity`]
without the need to specify the underlying function type in advance.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait QuantityFunction: dyn_clone::DynClone + Sync + Send + std::any::Any {
    /**
    Returns a quantity as a function of `conditions`. See the
    [`QuantityFunction`] trait docstring for examples.
    */
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64>;

    /**
    Returns `true` if `self` and `other` are identical and `false` otherwise.

    For a [`Sized`] type which implements [`PartialEq`], this function can be
    implemented as:

    ```ignore
    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
    ```

    If `Self` cannot be compared, this function should simply return `false`.
     */
    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool;
}

dyn_clone::clone_trait_object!(QuantityFunction);

/**
A quantity whose value can either be constant or a function of one or more other
quantities.

The value of (physical) quantities can depend on the values of other quantities.
This is often the case for quantities representing physical properties such as
e.g. the electric resistance of a conductor. This enum serves as a general
container for such quantities with the variant [`VarQuantity::Constant`] being
an optimization for the important case of a constant quantitity and with the
variant [`VarQuantity::Function`] covering all other cases via a
[`QuantityFunction`] trait object (wrapped in [`QuantityFunction`]). Due to the
generic design, it can also be used for dimensionless quantities which can be
represented by a simple [`f64`].

The value of the underlying quantity can be read out via the [`VarQuantity::get`]
method. It takes a slice of [`DynQuantity`] representing influencing factors,
for example the temperature in case of a resistance. If the enum variant is
constant, the value field is simply cloned, otherwise the [`QuantityFunction::call`]
function is called. This returns a [`DynQuantity<f64>`], which must be convertable
via [`TryFrom`] to `T` (enforced by trait bound). This dynamic approach is
chosen to make this enum serializable / deserializable (see section
[Features](#features)).

Even though the conversion from [`DynQuantity<f64>`] to `T` is fallible from the
perspective of the type system, in actual implementations it must be infallible
(i.e. the conversion must always succeed). This is done so `T` can be a
statically typed physical quantity (e.g. from the [uom](https://crates.io/crates/uom)
library), for which [`From<DynQuantity<f64>>`] can obviously not be implemented.
The conversion is checked once when constructing a [`QuantityFunction`] from a
[`QuantityFunction`] trait object by calling [`QuantityFunction::call`] with
`conditions = &[]`, but of course it is impossible to test all
potential values for `conditions`.

It is therefore up to the provider of the trait object to make sure that the
[`DynQuantity<f64>`] returned by [`QuantityFunction::call`] always has the same
[`Unit`]. If this is not the case, a default value for `T` will be returned.

# Examples

## f64 and statically typed physical quantities

This example shows how [`VarQuantity`] integrates with both [`f64`] and
[uom](https://crates.io/crates/uom) [`Quantity`](https://docs.rs/uom/latest/uom/si/struct.Quantity.html).
```
use var_quantity::{DynQuantity, PredefUnit, Unit, QuantityFunction, VarQuantity};
use var_quantity::uom::si::electrical_resistance::ohm;
use var_quantity::uom::si::f64::ElectricalResistance;

// =============================================================================
// Constant quantity with f64
let qt_const = VarQuantity::<f64>::Constant(2.0);

// Influencing factors
let infl1 = &[DynQuantity::new(6.0, PredefUnit::ElectricCurrent)];
let infl2 = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
    DynQuantity::new(20.0, PredefUnit::Temperature),
];

// Since this is a constant quantity, it returns always 2 regardless of the input.
assert_eq!(2.0, qt_const.get(&[]));
assert_eq!(2.0, qt_const.get(infl1));
assert_eq!(2.0, qt_const.get(infl2));

// =============================================================================
// Variable quantity

// A variable resistance: The resistance is 1 + temperature / 100.
// For the test, the serde feature is enabled, hence it is necessary to
// implement serialization and deserialization as well as #[typetag::serde].
// This is not needed if the feature is not enabled.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
struct ResistanceFunction;

#[typetag::serde]
impl QuantityFunction for ResistanceFunction {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut temperature = 0.0;
        let temperature_unit: Unit = PredefUnit::Temperature.into();
        for f in conditions.iter() {
            if f.unit == temperature_unit {
                temperature = f.value;
                break;
            }
        }
        return DynQuantity::new(1.0 + temperature / 100.0, PredefUnit::ElectricResistance);
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

let qt_var = VarQuantity::<ElectricalResistance>::new(ResistanceFunction {}).expect("type check successfull");

// Input infl2 contains a temperature and therefore influences the resistance.
assert_eq!(ElectricalResistance::new::<ohm>(1.0), qt_var.get(&[]));
assert_eq!(ElectricalResistance::new::<ohm>(1.0), qt_var.get(infl1));
assert_eq!(ElectricalResistance::new::<ohm>(1.2), qt_var.get(infl2));
```

## Unit mismatch

This example shows a violation of the assumption that the [`DynQuantity`] returned
by the [`QuantityFunction`] trait object is convertible to `T`.
```
use var_quantity::{DynQuantity, PredefUnit, QuantityFunction, VarQuantity};
use var_quantity::uom::si::electrical_conductance::siemens;
use var_quantity::uom::si::f64::{ElectricalResistance, ElectricalConductance};

#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
struct ResistanceFunction;

#[typetag::serde]
impl QuantityFunction for ResistanceFunction {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return DynQuantity::new(1.0, PredefUnit::ElectricResistance);
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

// Mismatch in type definition - catched during construction of QuantityFunction
assert!(VarQuantity::<ElectricalConductance>::new(ResistanceFunction {}).is_err());
```

# Features

If the `serde` feature is activated, this enum can be serialized and
deserialized (as untagged enum). The [`QuantityFunction`] trait object is
serialized / deserialized using [typetag](https://docs.rs/typetag/latest/typetag/).
This is also the reason why [`QuantityFunction::call`] returns a
[`DynQuantity<f64>`] instead of a generic type.
 */
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VarQuantity<T: QuantityF64> {
    /**
    Optimization for the common case of a constant quantity. This avoids
    dynamic dispatch when accessing the value.
     */
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    Constant(T),
    /**
    A quantity with arbitrary behaviour implemented by the contained
    [`QuantityFunction`] trait object.

    The output of the function must be convertible into a `T`, as outlined in the
    [`VarQuantity`] documentation. It is discouraged to construct this variant
    directly; use [`VarQuantity::new`] instead, which checks this constraint.
     */
    Function(Box<dyn QuantityFunction>),
}

impl<T: QuantityF64> std::fmt::Debug for VarQuantity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(arg0) => f.debug_tuple("Constant").field(arg0).finish(),
            Self::Function(_) => f.debug_tuple("Function").finish(),
        }
    }
}

impl<T: QuantityF64> PartialEq for VarQuantity<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Constant(l0), Self::Constant(r0)) => l0 == r0,
            (Self::Function(l0), Self::Function(r0)) => l0.dyn_eq(&**r0),
            _ => false,
        }
    }
}

impl<T: QuantityF64> VarQuantity<T> {
    /**
    Creates a new [`VarQuantity`] instance if the output of the given `function`
    can be converted into `T`.

    This is a convenience wrapper around the following steps:

    1. Check that the output of `function` can be converted into `T` by calling the
       function with an empty `conditions` slice. As stated in the
       [`QuantityFunction`] trait documentation, it is assumed that the function
       always returns quantities with the same unit.
    2. Box `function` as a trait object.
    3. Wrap the resulting trait object in [`VarQuantity<T>::Function`].

    Due to the additional unit check, it is recommended to use this function
    instead of directly constructing the [`VarQuantity<T>::Function`] variant.
    */
    pub fn new<F: QuantityFunction>(function: F) -> Result<Self, UnitsNotEqual> {
        // Check if the function produced the correct type for "default"
        // conditions.
        let output = function.call(&[]);
        if let Err(_) = T::try_from(output.into()) {
            return Err(UnitsNotEqual(T::unit_from_type(), output.unit));
        }
        return Ok(Self::Function(Box::new(function)));
    }

    /**
    Returns the contained value if `self` is [`VarQuantity::Constant`], or evaluates
    the contained [`QuantityFunction`] if `self` is [`VarQuantity::Function`].

    If the output of the [`QuantityFunction`] cannot be converted into `T`,
    [`T::default`] is returned instead.
    */
    pub fn get(&self, conditions: &[DynQuantity<f64>]) -> T {
        match self {
            Self::Constant(val) => val.clone(),
            Self::Function(fun) => T::try_from(fun.call(conditions).into()).unwrap_or_default(),
        }
    }

    /**
    Returns a reference to the underlying function if `self` is a
    [`VarQuantity::Function`].
     */
    pub fn function(&self) -> Option<&dyn QuantityFunction> {
        match self {
            VarQuantity::Constant(_) => return None,
            VarQuantity::Function(quantity_function) => return Some(quantity_function.as_ref()),
        }
    }
}

impl<T: QuantityF64> From<T> for VarQuantity<T> {
    fn from(value: T) -> Self {
        return Self::Constant(value);
    }
}

impl<T: QuantityF64> From<Box<dyn QuantityFunction>> for VarQuantity<T> {
    fn from(value: Box<dyn QuantityFunction>) -> Self {
        return Self::Function(value);
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::de::DeserializeOwned;

    use super::*;

    impl<'de, T> serde::Deserialize<'de> for VarQuantity<T>
    where
        T: DeserializeOwned + QuantityF64,
        <T as TryFrom<DynQuantity<Complex<f64>>>>::Error: std::fmt::Display,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use std::str::FromStr;

            #[derive(deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError)]
            enum InnerOrString<T> {
                Inner(T),
                #[cfg(feature = "from_str")]
                String(String),
            }

            let content: serde_value::Value = serde::Deserialize::deserialize(deserializer)?;

            // Try to deserialize as a quantity. If that fails, try to deserialize as a
            // function trait object
            match InnerOrString::<T>::deserialize(serde_value::ValueDeserializer::<D::Error>::new(
                content.clone(),
            )) {
                Ok(number_or_string) => match number_or_string {
                    InnerOrString::Inner(q) => return Ok(VarQuantity::Constant(q)),
                    InnerOrString::String(s) => {
                        let dq = DynQuantity::<Complex<f64>>::from_str(&s)
                            .map_err(serde::de::Error::custom)?;
                        let q = T::try_from(dq).map_err(serde::de::Error::custom)?;
                        return Ok(VarQuantity::Constant(q));
                    }
                },
                Err(_) => {
                    let function =
                        Box::<dyn QuantityFunction>::deserialize(serde_value::ValueDeserializer::<
                            D::Error,
                        >::new(
                            content
                        ))
                        .map_err(serde::de::Error::custom)?;
                    let output = function.call(&[]);
                    if let Err(_) = T::try_from(output.into()) {
                        return Err(serde::de::Error::custom(UnitsNotEqual(
                            T::unit_from_type(),
                            output.unit,
                        )));
                    }

                    Ok(VarQuantity::Function(function))
                }
            }
        }
    }
}

/**
A wrapper around a type implementing [`QuantityFunction`] which clamps the
output of [`QuantityFunction::call`] using the provided upper and lower limits.

If the `serde` feature is not enabled, [`ClampedQuantity`] implements
[`QuantityFunction`] for all `T` implementing [`QuantityFunction`]. If `serde`
is enabled, it is unfortately not possible to provide a generic implementation
due to the macro `#[typetag::serde]` not being able to deal with
generics. As a workaround, it is recommended to provide a simple custom
implementation for each concrete type in your own crate:

```ignore
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for ClampedQuantity<YourTypeHere> {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(conditions);
    }
}
```

This approach is used for all the implementors of [`QuantityFunction`] provided
with this crate.
 */
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClampedQuantity<T: QuantityFunction> {
    upper_limit: f64,
    lower_limit: f64,
    function: T,
}

impl<T: QuantityFunction> ClampedQuantity<T> {
    /**
    Checks if `upper_limit >= lower_limit` and returns a new instance of
    [`ClampedQuantity`] if true.
    */
    pub fn new(upper_limit: f64, lower_limit: f64, function: T) -> Option<Self> {
        if upper_limit < lower_limit {
            return None;
        }
        return Some(Self {
            upper_limit,
            lower_limit,
            function,
        });
    }

    /**
    Returns the underlying [`QuantityFunction`].
     */
    pub fn inner(&self) -> &T {
        return &self.function;
    }

    /**
    Returns the underlying [`QuantityFunction`] as a trait object.
     */
    pub fn inner_dyn(&self) -> &dyn QuantityFunction {
        return &self.function;
    }

    /// Returns the upper limit.
    pub fn upper_limit(&self) -> f64 {
        return self.upper_limit;
    }

    /// Returns the lower limit.
    pub fn lower_limit(&self) -> f64 {
        return self.lower_limit;
    }

    /**
    Clamps the output value of `T::call` using the provided upper and lower
    limits.

    This function is mainly here to simplify custom [`QuantityFunction`]
    implementations, see the [`ClampedQuantity`] docstring.
     */
    pub fn call_clamped(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut dyn_quantity = self.function.call(conditions);
        dyn_quantity.value = dyn_quantity.value.clamp(self.lower_limit, self.upper_limit);
        return dyn_quantity;
    }
}

// Only available if the serde feature is not active because deserialization of
// generic trait objects with typetag is not possible
#[cfg(not(feature = "serde"))]
impl<T: QuantityFunction + Clone + PartialEq> QuantityFunction for ClampedQuantity<T> {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(conditions);
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

/**
A helper function which filters the `conditions` for a quantity with
the type `match_for`. If a matching quantity is found, it is used as argument
for `with_matched` and the result is returned. Otherwise, the result of
`no_match()` is returned.

The main purpose of this function is to simplify writing unary functions. For
example, the [`QuantityFunction::call`] implementation of a linear function
can look like this:

```
use dyn_quantity::{DynQuantity, Unit};
use var_quantity::{filter_unary_function, QuantityFunction};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Linear {
    slope: f64,
    base_value: f64,
}

// Again, the macro annotation is just here because of the serde feature
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Linear {
    fn call(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            conditions,
            Unit::default(),
            |input| {
                DynQuantity::new(
                    self.base_value + self.slope * input.value,
                    Unit::default(),
                )
            },
            || DynQuantity::new(
                    self.base_value,
                    Unit::default(),
                ),
        );
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}
```
 */
pub fn filter_unary_function<F, G>(
    conditions: &[DynQuantity<f64>],
    match_for: Unit,
    with_matched: F,
    no_match: G,
) -> DynQuantity<f64>
where
    F: FnOnce(DynQuantity<f64>) -> DynQuantity<f64>,
    G: FnOnce() -> DynQuantity<f64>,
{
    for iq in conditions {
        if iq.unit == match_for {
            return with_matched(iq.clone());
        }
    }
    no_match()
}
