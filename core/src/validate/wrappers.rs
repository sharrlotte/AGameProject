use std::fmt;
use std::num::{NonZeroU32, NonZeroUsize};

use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

pub const fn non_zero_u32(value: u32) -> NonZeroU32 {
    match NonZeroU32::new(value) {
        Some(n) => n,
        None => panic!("value must be non-zero"),
    }
}

pub const fn non_zero_usize(value: usize) -> NonZeroUsize {
    match NonZeroUsize::new(value) {
        Some(n) => n,
        None => panic!("value must be non-zero"),
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct PositiveF32(f32);

impl PositiveF32 {
    pub fn new(value: f32) -> Result<Self, String> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(format!("must be finite and positive, got {value}"))
        }
    }

    pub const fn from_const(value: f32) -> Self {
        assert!(value.is_finite() && value > 0.0);
        Self(value)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for PositiveF32 {
    type Error = String;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveF32> for f32 {
    fn from(value: PositiveF32) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonNegativeF32(f32);

impl NonNegativeF32 {
    pub fn new(value: f32) -> Result<Self, String> {
        if value.is_finite() && value >= 0.0 {
            Ok(Self(value))
        } else {
            Err(format!("must be finite and non-negative, got {value}"))
        }
    }

    pub const fn from_const(value: f32) -> Self {
        assert!(value.is_finite() && value >= 0.0);
        Self(value)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for NonNegativeF32 {
    type Error = String;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NonNegativeF32> for f32 {
    fn from(value: NonNegativeF32) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct FiniteF32(f32);

impl FiniteF32 {
    pub fn new(value: f32) -> Result<Self, String> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(format!("must be finite, got {value}"))
        }
    }

    pub const fn from_const(value: f32) -> Self {
        assert!(value.is_finite());
        Self(value)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for FiniteF32 {
    type Error = String;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FiniteF32> for f32 {
    fn from(value: FiniteF32) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "Vec2", into = "Vec2")]
pub struct FiniteVec2(Vec2);

impl FiniteVec2 {
    pub fn new(value: Vec2) -> Result<Self, String> {
        if value.x.is_finite() && value.y.is_finite() {
            Ok(Self(value))
        } else {
            Err(format!("vector components must be finite, got {value}"))
        }
    }

    pub const fn from_const(x: f32, y: f32) -> Self {
        assert!(x.is_finite() && y.is_finite());
        Self(Vec2::new(x, y))
    }

    pub const fn get(self) -> Vec2 {
        self.0
    }
}

impl TryFrom<Vec2> for FiniteVec2 {
    type Error = String;
    fn try_from(value: Vec2) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FiniteVec2> for Vec2 {
    fn from(value: FiniteVec2) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct SkillLevel(u8);

impl SkillLevel {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(10);

    pub fn new(value: u8) -> Result<Self, String> {
        if (1..=10).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("skill level must be in 1..=10, got {value}"))
        }
    }

    pub const fn from_const(value: u8) -> Self {
        assert!(value >= 1 && value <= 10);
        Self(value)
    }

    pub fn saturating(value: i16) -> Self {
        Self(value.clamp(Self::MIN.0 as i16, Self::MAX.0 as i16) as u8)
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl Default for SkillLevel {
    fn default() -> Self {
        Self(5)
    }
}

impl TryFrom<u8> for SkillLevel {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SkillLevel> for u8 {
    fn from(value: SkillLevel) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct BoostFactor(f32);

impl BoostFactor {
    pub fn new(value: f32) -> Result<Self, String> {
        if value.is_finite() && value >= 1.0 {
            Ok(Self(value))
        } else {
            Err(format!("boost factor must be finite and >= 1.0, got {value}"))
        }
    }

    pub const fn from_const(value: f32) -> Self {
        assert!(value.is_finite() && value >= 1.0);
        Self(value)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

impl Default for BoostFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl TryFrom<f32> for BoostFactor {
    type Error = String;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<BoostFactor> for f32 {
    fn from(value: BoostFactor) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(try_from = "i8", into = "i8")]
pub struct PositiveI8(i8);

impl PositiveI8 {
    pub fn new(value: i8) -> Result<Self, String> {
        if value >= 1 {
            Ok(Self(value))
        } else {
            Err(format!("must be at least 1, got {value}"))
        }
    }

    pub const fn from_const(value: i8) -> Self {
        assert!(value >= 1);
        Self(value)
    }

    pub const fn get(self) -> i8 {
        self.0
    }
}

impl TryFrom<i8> for PositiveI8 {
    type Error = String;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveI8> for i8 {
    fn from(value: PositiveI8) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(try_from = "(T, T)", into = "(T, T)")]
pub struct OrderedRange<T: PartialOrd + Copy + fmt::Debug> {
    min: T,
    max: T,
}

impl<T: PartialOrd + Copy + fmt::Debug> OrderedRange<T> {
    pub fn new(min: T, max: T) -> Result<Self, String> {
        if min <= max {
            Ok(Self { min, max })
        } else {
            Err(format!("range min ({min:?}) must be <= max ({max:?})"))
        }
    }

    pub const fn min(self) -> T {
        self.min
    }

    pub const fn max(self) -> T {
        self.max
    }
}

impl OrderedRange<PositiveF32> {
    pub const fn from_const(min: f32, max: f32) -> Self {
        let min = PositiveF32::from_const(min);
        let max = PositiveF32::from_const(max);
        assert!(min.get() <= max.get());
        Self { min, max }
    }
}

impl OrderedRange<NonNegativeF32> {
    pub const fn from_const(min: f32, max: f32) -> Self {
        let min = NonNegativeF32::from_const(min);
        let max = NonNegativeF32::from_const(max);
        assert!(min.get() <= max.get());
        Self { min, max }
    }
}

impl OrderedRange<NonZeroU32> {
    pub const fn from_const(min: u32, max: u32) -> Self {
        assert!(min >= 1 && min <= max);
        Self {
            min: non_zero_u32(min),
            max: non_zero_u32(max),
        }
    }
}

impl<T: PartialOrd + Copy + fmt::Debug> TryFrom<(T, T)> for OrderedRange<T> {
    type Error = String;
    fn try_from((min, max): (T, T)) -> Result<Self, Self::Error> {
        Self::new(min, max)
    }
}

impl<T: PartialOrd + Copy + fmt::Debug> From<OrderedRange<T>> for (T, T) {
    fn from(range: OrderedRange<T>) -> Self {
        (range.min, range.max)
    }
}

pub trait IsEmpty {
    fn is_empty_value(&self) -> bool;
}

impl IsEmpty for String {
    fn is_empty_value(&self) -> bool {
        self.trim().is_empty()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty_value(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NonEmpty<T>(T);

impl<T: IsEmpty> NonEmpty<T> {
    pub fn new(value: T) -> Result<Self, String> {
        if value.is_empty_value() {
            Err("value must not be empty".to_string())
        } else {
            Ok(Self(value))
        }
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> Serialize for NonEmpty<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + IsEmpty> Deserialize<'de> for NonEmpty<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl<T> NonEmpty<Vec<T>> {
    pub fn single(item: T) -> Self {
        Self(vec![item])
    }
}

impl NonEmpty<String> {
    pub fn from_static_str(value: &'static str) -> Self {
        assert!(!value.trim_ascii().is_empty());
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_bounds_reject_invalid_values() {
        assert!(PositiveF32::new(1.0).is_ok());
        assert!(PositiveF32::new(0.0).is_err());
        assert!(PositiveF32::new(-1.0).is_err());
        assert!(PositiveF32::new(f32::NAN).is_err());

        assert!(NonNegativeF32::new(0.0).is_ok());
        assert!(NonNegativeF32::new(-0.01).is_err());

        assert!(FiniteF32::new(-5.0).is_ok());
        assert!(FiniteF32::new(f32::INFINITY).is_err());

        assert!(FiniteVec2::new(Vec2::new(1.0, -2.0)).is_ok());
        assert!(FiniteVec2::new(Vec2::new(f32::NAN, 0.0)).is_err());
        assert!(FiniteVec2::new(Vec2::new(0.0, f32::INFINITY)).is_err());

        assert!(SkillLevel::new(1).is_ok());
        assert!(SkillLevel::new(10).is_ok());
        assert!(SkillLevel::new(0).is_err());
        assert!(SkillLevel::new(11).is_err());
        assert_eq!(SkillLevel::saturating(-5), SkillLevel::MIN);
        assert_eq!(SkillLevel::saturating(25), SkillLevel::MAX);

        assert!(BoostFactor::new(1.0).is_ok());
        assert!(BoostFactor::new(0.99).is_err());

        assert!(PositiveI8::new(1).is_ok());
        assert!(PositiveI8::new(0).is_err());
        assert!(PositiveI8::new(-1).is_err());

        assert!(OrderedRange::new(1, 2).is_ok());
        assert!(OrderedRange::new(2, 2).is_ok());
        assert!(OrderedRange::new(3, 2).is_err());

        assert!(NonEmpty::new("hello".to_string()).is_ok());
        assert!(NonEmpty::new("   ".to_string()).is_err());
        assert!(NonEmpty::new(vec![1]).is_ok());
        assert!(NonEmpty::<Vec<i32>>::new(vec![]).is_err());
    }
}
