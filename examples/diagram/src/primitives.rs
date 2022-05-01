use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_traits::{One, ToPrimitive, Zero};

pub type FixedType = fixed::types::I32F32;

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Unit(pub FixedType);

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Hash for Unit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Eq for Unit {}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits().eq(&other.0.to_bits())
    }
}

impl Ord for Unit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Unit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.to_bits().partial_cmp(&other.0.to_bits())
    }
}

impl ToPrimitive for Unit {
    fn to_i64(&self) -> Option<i64> {
        self.0.checked_to_num::<i64>()
    }

    fn to_u64(&self) -> Option<u64> {
        self.0.checked_to_num::<u64>()
    }
}

impl From<i32> for Unit {
    fn from(v: i32) -> Unit {
        FixedType::checked_from_num(v).map(|result| Unit(result)).unwrap()
    }
}

impl From<u16> for Unit {
    fn from(v: u16) -> Unit {
        FixedType::checked_from_num(v).map(|result| Unit(result)).unwrap()
    }
}

impl From<f64> for Unit {
    fn from(v: f64) -> Self {
        FixedType::checked_from_num(v).map(|result| Unit(result)).unwrap()
    }
}

impl num_traits::NumCast for Unit {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        match n.to_i64() {
            Some(i) => FixedType::checked_from_num(i).map(|result| Unit(result)),
            None => n
                .to_u64()
                .and_then(FixedType::checked_from_num)
                .map(|result| Unit(result)),
        }
    }
}

impl Neg for Unit {
    type Output = Unit;

    fn neg(self) -> Self::Output {
        Unit(-self.0.neg())
    }
}

impl Zero for Unit {
    fn zero() -> Self {
        Unit(FixedType::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl Add for Unit {
    type Output = Unit;

    fn add(self, rhs: Unit) -> Unit {
        Unit(self.0.add(rhs.0))
    }
}

impl One for Unit {
    fn one() -> Self {
        Unit(FixedType::one())
    }
}

impl Mul for Unit {
    type Output = Unit;

    fn mul(self, rhs: Unit) -> Unit {
        Unit(self.0.mul(rhs.0))
    }
}

impl Sub for Unit {
    type Output = Unit;

    fn sub(self, rhs: Unit) -> Unit {
        Unit(self.0.sub(rhs.0))
    }
}

impl Div for Unit {
    type Output = Unit;

    fn div(self, rhs: Unit) -> Unit {
        Unit(self.0.div(rhs.0))
    }
}

impl Rem for Unit {
    type Output = Unit;

    fn rem(self, rhs: Unit) -> Unit {
        Unit(self.0.rem(rhs.0))
    }
}

impl num_traits::Num for Unit {
    type FromStrRadixErr = fixed::RadixParseFixedError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        FixedType::from_str_radix(str, radix).map(|result| Unit(result))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct HorizontalSegment(pub geo::Line<Unit>);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct VerticalSegment(pub geo::Line<Unit>);

impl From<geo::Line<Unit>> for HorizontalSegment {
    fn from(line: geo::Line<Unit>) -> Self {
        assert_eq!(line.start.y, line.end.y);
        Self(line)
    }
}

impl From<geo::Line<Unit>> for VerticalSegment {
    fn from(line: geo::Line<Unit>) -> Self {
        assert_eq!(line.start.x, line.end.x);
        Self(line)
    }
}

// impl proptest::arbitrary::Arbitrary for HorizontalSegment {
//     type Parameters = ();
//     fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
//         (proptest::arbitrary::any<Unit>::(), proptest::arbitrary::any<Unit>::()).prop_map(|(x, y)|) {
//
//         }
//         todo!()
//     }
//
//     type Strategy = proptest::strategy::BoxedStrategy<Self>;
// }

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PortNumber(pub u16);

/// Ports represents how many connections are on the top, right, bottom, and left of a GeomBox.
/// 1 is default and means you have north, east, south, and west points in the middle of each
/// side. Any or all can be zero, meaning no connectors. Cannot be negative.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ports {
    pub top: PortNumber,
    pub right: PortNumber,
    pub bottom: PortNumber,
    pub left: PortNumber,
}

impl Ports {
    pub fn new<T: num_traits::NumCast>(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top: PortNumber(num::cast(top).unwrap()),
            right: PortNumber(num::cast(right).unwrap()),
            bottom: PortNumber(num::cast(bottom).unwrap()),
            left: PortNumber(num::cast(left).unwrap()),
        }
    }
}

impl Default for Ports {
    fn default() -> Self {
        Ports {
            top: PortNumber(1),
            right: PortNumber(1),
            bottom: PortNumber(1),
            left: PortNumber(1),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Padding {
    pub top: Unit,
    pub right: Unit,
    pub bottom: Unit,
    pub left: Unit,
}

impl Padding {
    pub fn new_uniform<T: Into<Unit> + Clone + Copy>(amount: T) -> Self {
        Padding {
            top: amount.into(),
            right: amount.into(),
            bottom: amount.into(),
            left: amount.into(),
        }
    }
}
