//! Defines the subtract arithmetic kernels for Decimal `PrimitiveArrays`.

use crate::compute::arithmetics::basic::check_same_len;
use crate::{
    array::{Array, PrimitiveArray},
    buffer::Buffer,
    compute::{
        arithmetics::{ArrayCheckedSub, ArraySaturatingSub, ArraySub},
        arity::{binary, binary_checked},
        utils::combine_validities,
    },
    datatypes::DataType,
    error::{ArrowError, Result},
};

use super::{adjusted_precision_scale, get_parameters, max_value, number_digits};

/// Subtract two decimal primitive arrays with the same precision and scale. If
/// the precision and scale is different, then an InvalidArgumentError is
/// returned. This function panics if the subtracted numbers result in a number
/// smaller than the possible number for the selected precision.
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(1i128), Some(1i128), None, Some(2i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(1i128), Some(2i128), None, Some(2i128)]).to(DataType::Decimal(5, 2));
///
/// let result = sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(0i128), Some(-1i128), None, Some(0i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn sub(lhs: &PrimitiveArray<i128>, rhs: &PrimitiveArray<i128>) -> Result<PrimitiveArray<i128>> {
    let (precision, _) = get_parameters(lhs.data_type(), rhs.data_type())?;

    let max = max_value(precision);

    let op = move |a, b| {
        let res: i128 = a - b;

        assert!(
            res.abs() <= max,
            "Overflow in subtract presented for precision {}",
            precision
        );

        res
    };

    binary(lhs, rhs, lhs.data_type().clone(), op)
}

/// Saturated subtraction of two decimal primitive arrays with the same
/// precision and scale. If the precision and scale is different, then an
/// InvalidArgumentError is returned. If the result from the sum is smaller
/// than the possible number with the selected precision then the resulted
/// number in the arrow array is the minimum number for the selected precision.
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::saturating_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(-99000i128), Some(11100i128), None, Some(22200i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(01000i128), Some(22200i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// let result = saturating_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(-99999i128), Some(-11100i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn saturating_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    let (precision, _) = get_parameters(lhs.data_type(), rhs.data_type())?;

    let max = max_value(precision);

    let op = move |a, b| {
        let res: i128 = a - b;

        match res {
            res if res.abs() > max => {
                if res > 0 {
                    max
                } else {
                    -max
                }
            }
            _ => res,
        }
    };

    binary(lhs, rhs, lhs.data_type().clone(), op)
}

// Implementation of ArraySub trait for PrimitiveArrays
impl ArraySub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    fn sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self> {
        sub(self, rhs)
    }
}

// Implementation of ArrayCheckedSub trait for PrimitiveArrays
impl ArrayCheckedSub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    fn checked_sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self> {
        checked_sub(self, rhs)
    }
}

// Implementation of ArraySaturatingSub trait for PrimitiveArrays
impl ArraySaturatingSub<PrimitiveArray<i128>> for PrimitiveArray<i128> {
    fn saturating_sub(&self, rhs: &PrimitiveArray<i128>) -> Result<Self> {
        saturating_sub(self, rhs)
    }
}
/// Checked subtract of two decimal primitive arrays with the same precision
/// and scale. If the precision and scale is different, then an
/// InvalidArgumentError is returned. If the result from the sub is larger than
/// the possible number with the selected precision (overflowing), then the
/// validity for that index is changed to None
///
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::checked_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(-99000i128), Some(11100i128), None, Some(22200i128)]).to(DataType::Decimal(5, 2));
/// let b = PrimitiveArray::from([Some(01000i128), Some(22200i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// let result = checked_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([None, Some(-11100i128), None, Some(11100i128)]).to(DataType::Decimal(5, 2));
///
/// assert_eq!(result, expected);
/// ```
pub fn checked_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    let (precision, _) = get_parameters(lhs.data_type(), rhs.data_type())?;

    let max = max_value(precision);

    let op = move |a, b| {
        let res: i128 = a - b;

        match res {
            res if res.abs() > max => None,
            _ => Some(res),
        }
    };

    binary_checked(lhs, rhs, lhs.data_type().clone(), op)
}

/// Adaptive subtract of two decimal primitive arrays with different precision
/// and scale. If the precision and scale is different, then the smallest scale
/// and precision is adjusted to the largest precision and scale. If during the
/// addition one of the results is smaller than the min possible value, the
/// result precision is changed to the precision of the min value
///
/// ```nocode
///  99.9999 -> 6, 4
/// -00.0001 -> 6, 4
/// -----------------
/// 100.0000 -> 7, 4
/// ```
/// # Examples
/// ```
/// use arrow2::compute::arithmetics::decimal::sub::adaptive_sub;
/// use arrow2::array::PrimitiveArray;
/// use arrow2::datatypes::DataType;
///
/// let a = PrimitiveArray::from([Some(99_9999i128)]).to(DataType::Decimal(6, 4));
/// let b = PrimitiveArray::from([Some(-00_0001i128)]).to(DataType::Decimal(6, 4));
/// let result = adaptive_sub(&a, &b).unwrap();
/// let expected = PrimitiveArray::from([Some(100_0000i128)]).to(DataType::Decimal(7, 4));
///
/// assert_eq!(result, expected);
/// ```
pub fn adaptive_sub(
    lhs: &PrimitiveArray<i128>,
    rhs: &PrimitiveArray<i128>,
) -> Result<PrimitiveArray<i128>> {
    check_same_len(lhs, rhs)?;

    if let (DataType::Decimal(lhs_p, lhs_s), DataType::Decimal(rhs_p, rhs_s)) =
        (lhs.data_type(), rhs.data_type())
    {
        // The resulting precision is mutable because it could change while
        // looping through the iterator
        let (mut res_p, res_s, diff) = adjusted_precision_scale(*lhs_p, *lhs_s, *rhs_p, *rhs_s);

        let shift = 10i128.pow(diff as u32);
        let mut max = max_value(res_p);

        let iter = lhs.values().iter().zip(rhs.values().iter()).map(|(l, r)| {
            // Based on the array's scales one of the arguments in the sum has to be shifted
            // to the left to match the final scale
            let res: i128 = if lhs_s > rhs_s {
                l - r * shift
            } else {
                l * shift - r
            };

            // The precision of the resulting array will change if one of the
            // subtraction during the iteration produces a value bigger than the
            // possible value for the initial precision

            //  -99.9999 -> 6, 4
            //   00.0001 -> 6, 4
            // -----------------
            // -100.0000 -> 7, 4
            if res.abs() > max {
                res_p = number_digits(res);
                max = max_value(res_p);
            }

            res
        });
        let values = Buffer::from_trusted_len_iter(iter);

        let validity = combine_validities(lhs.validity(), rhs.validity());

        Ok(PrimitiveArray::<i128>::from_data(
            DataType::Decimal(res_p, res_s),
            values,
            validity,
        ))
    } else {
        Err(ArrowError::InvalidArgumentError(
            "Incorrect data type for the array".to_string(),
        ))
    }
}
