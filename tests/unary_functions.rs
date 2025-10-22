use approx;
use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use indoc::indoc;
use var_quantity::{QuantityFunction, unary::*};

#[test]
fn test_exponential() {
    // No units
    {
        let term0 = ExpTerm {
            amplitude: 2.0.into(),
            exponent: 2.0.into(),
        };
        let term1 = ExpTerm {
            amplitude: (-3.0).into(),
            exponent: 0.0.into(),
        };
        let fun = Exponential::new(vec![term0, term1]).unwrap();

        assert_eq!(fun.call(&[0.0.into()]).value, -1.0);
        approx::assert_abs_diff_eq!(fun.call(&[1.0.into()]).value, 11.77811, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(fun.call(&[2.0.into()]).value, 106.1963, epsilon = 0.0001);
    }

    // With units
    {
        let per_ampere = Unit::from(PredefUnit::ElectricCurrent).powi(-1);
        let term0 = ExpTerm {
            amplitude: DynQuantity::new(2.0, PredefUnit::ElectricVoltage),
            exponent: DynQuantity::new(2.0, per_ampere),
        };
        let term1 = ExpTerm {
            amplitude: DynQuantity::new(-3.0, PredefUnit::ElectricVoltage),
            exponent: DynQuantity::new(0.0, per_ampere),
        };
        let fun = Exponential::new(vec![term0, term1]).unwrap();

        // Matching unit
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            -1.0
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            11.77811,
            epsilon = 0.0001
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::ElectricCurrent)])
                .value,
            106.1963,
            epsilon = 0.0001
        );
    }
}

#[test]
fn test_first_order_taylor() {
    // No units
    {
        let fun = FirstOrderTaylor::new(2.5.into(), 2.0.into(), 0.5.into()).unwrap();
        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 0.0);
        assert_eq!(fun.call(&[1.5.into()]).value, 7.5);
    }

    // With units
    {
        let fun = FirstOrderTaylor::new(
            DynQuantity::new(2.5, PredefUnit::Power),
            DynQuantity::new(2.0, PredefUnit::ElectricVoltage),
            DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
        )
        .unwrap();

        assert_eq!(
            fun.call(&[DynQuantity::new(0.5, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            0.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(1.5, PredefUnit::ElectricCurrent)])
                .value,
            7.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.5);
        assert_eq!(fun.call(&[1.5.into()]).value, 2.5);
    }

    // Unit mismatch
    {
        assert!(
            FirstOrderTaylor::new(
                DynQuantity::new(2.5, PredefUnit::Power),
                DynQuantity::new(2.0, PredefUnit::ElectricVoltage),
                DynQuantity::new(0.5, PredefUnit::Force),
            )
            .is_err()
        );
        assert!(
            FirstOrderTaylor::new(
                DynQuantity::new(2.5, PredefUnit::Power),
                DynQuantity::new(2.0, PredefUnit::Force),
                DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
            )
            .is_err()
        );
        assert!(
            FirstOrderTaylor::new(
                DynQuantity::new(2.5, PredefUnit::Force),
                DynQuantity::new(2.0, PredefUnit::ElectricVoltage),
                DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
            )
            .is_err()
        );
    }
}

#[test]
fn test_linear() {
    // No units
    {
        let fun = Linear::new(0.5.into(), (-3.0).into());
        assert_eq!(fun.call(&[2.0.into()]).value, -2.0);
    }

    // With units
    {
        let fun = Linear::new(
            DynQuantity::new(2.0, PredefUnit::ElectricVoltage),
            DynQuantity::new(0.5, PredefUnit::Power),
        );

        assert_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(2.5, PredefUnit::ElectricCurrent)])
                .value,
            5.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[1.0.into()]).value, 0.5);
        assert_eq!(fun.call(&[2.0.into()]).value, 0.5);
    }
}

#[test]
fn test_polynomial() {
    // No units
    {
        // 3.0x + 2.0
        let fun = Polynomial::new(vec![3.0.into(), 2.0.into()]).unwrap();
        assert_eq!(fun.call(&[2.0.into()]).value, 8.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // -1.0x² + 3.0x + 2.0
        let fun = Polynomial::new(vec![(-1.0).into(), 3.0.into(), 2.0.into()]).unwrap();
        assert_eq!(fun.call(&[2.0.into()]).value, 4.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);
    }

    // With units
    {
        let fun = Polynomial::new(vec![
            DynQuantity::new(-1.0, PredefUnit::Length),
            DynQuantity::new(3.0, PredefUnit::Area),
            DynQuantity::new(2.0, PredefUnit::Volume),
        ])
        .unwrap();

        // Influencing factor does not have the right type
        assert_eq!(fun.call(&[2.0.into()]).value, 2.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // Influencing factor has the right type (length)
        assert_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::Length)]).value,
            4.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::Length)]).value,
            2.0
        );
    }

    // Unit mismatch
    {
        assert!(
            Polynomial::new(vec![
                DynQuantity::new(-1.0, PredefUnit::Length),
                DynQuantity::new(3.0, PredefUnit::Area),
                DynQuantity::new(2.0, PredefUnit::ElectricCurrent),
            ],)
            .is_err()
        );
        assert!(
            Polynomial::new(vec![
                DynQuantity::new(-1.0, PredefUnit::Length),
                DynQuantity::new(3.0, PredefUnit::ElectricCurrent),
                DynQuantity::new(2.0, PredefUnit::Volume),
            ],)
            .is_err()
        );
        assert!(
            Polynomial::new(vec![
                DynQuantity::new(-1.0, PredefUnit::ElectricCurrent),
                DynQuantity::new(3.0, PredefUnit::Area),
                DynQuantity::new(2.0, PredefUnit::Volume),
            ],)
            .is_err()
        );
    }
}

// =================================================
// Serde

#[test]
fn test_exponential_serde() {
    // No units
    {
        let yaml = indoc! {"
        ---
        terms:
          - amplitude: 2.0
            exponent: 2.0
          - amplitude: -3.0
            exponent: 0.0
        "};

        let fun: Exponential = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(fun.call(&[0.0.into()]).value, -1.0);
        approx::assert_abs_diff_eq!(fun.call(&[1.0.into()]).value, 11.77811, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(fun.call(&[2.0.into()]).value, 106.1963, epsilon = 0.0001);
        assert_eq!(fun.call(&[0.0.into()]).unit, PredefUnit::None.into());

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Exponential = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(fun.call(&[0.0.into()]).value, -1.0);
        approx::assert_abs_diff_eq!(fun.call(&[1.0.into()]).value, 11.77811, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(fun.call(&[2.0.into()]).value, 106.1963, epsilon = 0.0001);
        assert_eq!(fun.call(&[0.0.into()]).unit, PredefUnit::None.into());
    }

    // With units
    {
        let yaml = indoc! {"
        ---
        terms:
          - amplitude: 2.0 V
            exponent: 2.0 / A
          - amplitude: -3.0 V
            exponent: 0.0 / A
        "};

        let fun: Exponential = serde_yaml::from_str(yaml).unwrap();

        // Mismatching input unit
        assert_eq!(fun.call(&[0.0.into()]).value, -1.0);
        assert_eq!(fun.call(&[1.0.into()]).value, -1.0);
        assert_eq!(fun.call(&[2.0.into()]).value, -1.0);

        // Matching unit
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            -1.0
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            11.77811,
            epsilon = 0.0001
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::ElectricCurrent)])
                .value,
            106.1963,
            epsilon = 0.0001
        );

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Exponential = serde_yaml::from_str(&serialized).unwrap();

        // Mismatching input unit
        assert_eq!(fun.call(&[0.0.into()]).value, -1.0);
        assert_eq!(fun.call(&[1.0.into()]).value, -1.0);
        assert_eq!(fun.call(&[2.0.into()]).value, -1.0);

        // Matching unit
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            -1.0
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            11.77811,
            epsilon = 0.0001
        );
        approx::assert_abs_diff_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::ElectricCurrent)])
                .value,
            106.1963,
            epsilon = 0.0001
        );
    }
}

#[test]
fn test_first_order_taylor_serde() {
    // No units
    {
        let yaml = indoc! {"
        ---
        base_value: 2.5
        slope: 2.0
        expansion_point: 0.5
        "};
        let fun: FirstOrderTaylor = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 0.0);
        assert_eq!(fun.call(&[1.5.into()]).value, 7.5);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: FirstOrderTaylor = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 0.0);
        assert_eq!(fun.call(&[1.5.into()]).value, 7.5);
    }

    // With units
    {
        let yaml = indoc! {"
        ---
        base_value: 2.5 W
        slope: 2.0 V
        expansion_point: 0.5 A
        "};
        let fun: FirstOrderTaylor = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(
            fun.call(&[DynQuantity::new(0.5, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            0.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(1.5, PredefUnit::ElectricCurrent)])
                .value,
            7.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.5);
        assert_eq!(fun.call(&[1.5.into()]).value, 2.5);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: FirstOrderTaylor = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(
            fun.call(&[DynQuantity::new(0.5, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::ElectricCurrent)])
                .value,
            0.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(1.5, PredefUnit::ElectricCurrent)])
                .value,
            7.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[0.5.into()]).value, 2.5);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.5);
        assert_eq!(fun.call(&[1.5.into()]).value, 2.5);
    }
}

#[test]
fn test_linear_serde() {
    // No units
    {
        let yaml = indoc! {"
        ---
        slope: 0.5
        base_value: -3.0
        "};
        let fun: Linear = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(fun.call(&[2.0.into()]).value, -2.0);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Linear = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(fun.call(&[2.0.into()]).value, -2.0);
    }

    // With units
    {
        let yaml = indoc! {"
        ---
        slope: 2.0 V
        base_value: 0.5 W
        "};
        let fun: Linear = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(2.5, PredefUnit::ElectricCurrent)])
                .value,
            5.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[1.0.into()]).value, 0.5);
        assert_eq!(fun.call(&[2.0.into()]).value, 0.5);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Linear = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(
            fun.call(&[DynQuantity::new(1.0, PredefUnit::ElectricCurrent)])
                .value,
            2.5
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(2.5, PredefUnit::ElectricCurrent)])
                .value,
            5.5
        );

        // Input type does not match influencing quantity -> return base value
        assert_eq!(fun.call(&[1.0.into()]).value, 0.5);
        assert_eq!(fun.call(&[2.0.into()]).value, 0.5);
    }
}

#[test]
fn test_polynomial_serde() {
    // No units
    {
        // 3.0x + 2.0
        let yaml = indoc! {"
        ---
        coefficients:
          - 3.0
          - 2.0
        "};
        let fun: Polynomial = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(fun.call(&[2.0.into()]).value, 8.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Polynomial = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(fun.call(&[2.0.into()]).value, 8.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // -1.0x² + 3.0x + 2.0
        let yaml = indoc! {"
        ---
        coefficients:
          - -1.0
          - 3.0
          - 2.0
        "};
        let fun: Polynomial = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fun.call(&[2.0.into()]).value, 4.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Polynomial = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(fun.call(&[2.0.into()]).value, 4.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);
    }

    // With units
    {
        let yaml = indoc! {"
        ---
        coefficients:
          - -1.0 m
          - 3.0 m^2
          - 2.0 m^3
        "};

        let fun: Polynomial = serde_yaml::from_str(yaml).unwrap();

        // Influencing factor does not have the right type
        assert_eq!(fun.call(&[2.0.into()]).value, 2.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // Influencing factor has the right type (length)
        assert_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::Length)]).value,
            4.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::Length)]).value,
            2.0
        );

        // Serialize again and deserialize, then run the tests again
        let serialized = serde_yaml::to_string(&fun).unwrap();
        let fun: Polynomial = serde_yaml::from_str(&serialized).unwrap();

        // Influencing factor does not have the right type
        assert_eq!(fun.call(&[2.0.into()]).value, 2.0);
        assert_eq!(fun.call(&[0.0.into()]).value, 2.0);

        // Influencing factor has the right type (length)
        assert_eq!(
            fun.call(&[DynQuantity::new(2.0, PredefUnit::Length)]).value,
            4.0
        );
        assert_eq!(
            fun.call(&[DynQuantity::new(0.0, PredefUnit::Length)]).value,
            2.0
        );
    }
}
