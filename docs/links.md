var_quantity
============

<!-- This file has ben generated with build.rs by concatenating docs/links.md,
docs/main.md and (if available docs/end.md). Do not modify this file, instead
modify the components. -->

[`VarQuantity`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/enum.VarQuantity.html
[`VarQuantity::Constant`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/enum.VarQuantity.html#variant.Constant
[`VarQuantity::Function`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/enum.VarQuantity.html#variant.Function
[`VarQuantity::get`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/enum.VarQuantity.html#method.get
[`QuantityFunction`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/struct.QuantityFunction.html
[`QuantityFunction::call`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/struct.QuantityFunction.html#method.call
[`QuantityFunction::new`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/struct.QuantityFunction.html#method.new
[`IsQuantityFunction`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/trait.IsQuantityFunction.html
[`IsQuantityFunction::call`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/trait.IsQuantityFunction.html#tymethod.call
[`filter_unary_function`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/fn.filter_unary_function.html
[`Polynomial`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/unary/struct.Polynomial.html
[`unary`]: https://docs.rs/var_quantity/{{VERSION}}/var_quantity/unary.html
[typetag]: (https://docs.rs/typetag/latest/typetag/)
[uom]: (https://crates.io/crates/uom)

[![Documentation](https://docs.rs/var_quantity/badge.svg)](https://docs.rs/var_quantity)

This crate is an extension of [dyn_quantity](https://crates.io/crates/dyn_quantity)
and provides an interface for defining variable quantities whose value is a
(pure) function of other quantities.

The full API documentation is available at <https://docs.rs/var_quantity/{{VERSION}}/var_quantity>.

