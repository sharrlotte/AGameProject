
pub trait Validate {
    type Context: ?Sized;
    type Error;

    fn validate(&self, ctx: &Self::Context) -> Result<(), Self::Error>;
}

pub mod check {
    use std::fmt::Display;
    
    pub fn positive(field: &'static str, val: f32) -> Result<(), String> {
        if val.is_finite() && val > 0.0 {
            Ok(())
        } else {
            Err(format!("{field} must be positive, got {val}"))
        }
    }

    pub fn non_negative(field: &'static str, val: f32) -> Result<(), String> {
        if val.is_finite() && val >= 0.0 {
            Ok(())
        } else {
            Err(format!("{field} must be non-negative, got {val}"))
        }
    }

    pub fn finite(field: &'static str, val: f32) -> Result<(), String> {
        if val.is_finite() {
            Ok(())
        } else {
            Err(format!("{field} must be finite, got {val}"))
        }
    }

    pub fn range_ordered(
        min_name: &'static str,
        min: f32,
        max_name: &'static str,
        max: f32,
    ) -> Result<(), String> {
        if max >= min {
            Ok(())
        } else {
            Err(format!("{max_name} ({max}) must be at least {min_name} ({min})"))
        }
    }

    pub fn bounded_int<T: PartialOrd + Display + Copy>(
        field: &'static str,
        val: T,
        min: T,
        max: T,
    ) -> Result<(), String> {
        if val >= min && val <= max {
            Ok(())
        } else {
            Err(format!("{field} must be between {min} and {max}, got {val}"))
        }
    }

    pub fn at_least<T: PartialOrd + Display + Copy>(
        field: &'static str,
        val: T,
        min: T,
    ) -> Result<(), String> {
        if val >= min {
            Ok(())
        } else {
            Err(format!("{field} must be at least {min}, got {val}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive() {
        assert!(check::positive("val", 1.0).is_ok());
        assert!(check::positive("val", 0.0).is_err());
        assert!(check::positive("val", -1.0).is_err());
        assert!(check::positive("val", f32::NAN).is_err());
        assert!(check::positive("val", f32::INFINITY).is_err());
    }

    #[test]
    fn test_non_negative() {
        assert!(check::non_negative("val", 0.0).is_ok());
        assert!(check::non_negative("val", 1.0).is_ok());
        assert!(check::non_negative("val", -0.01).is_err());
        assert!(check::non_negative("val", f32::NAN).is_err());
    }

    #[test]
    fn test_range_ordered() {
        assert!(check::range_ordered("min", 1.0, "max", 2.0).is_ok());
        assert!(check::range_ordered("min", 2.0, "max", 2.0).is_ok());
        assert!(check::range_ordered("min", 2.1, "max", 2.0).is_err());
    }

    #[test]
    fn test_bounded_int() {
        assert!(check::bounded_int("level", 5, 1, 10).is_ok());
        assert!(check::bounded_int("level", 0, 1, 10).is_err());
        assert!(check::bounded_int("level", 11, 1, 10).is_err());
    }

    #[test]
    fn test_at_least() {
        assert!(check::at_least("count", 1, 1).is_ok());
        assert!(check::at_least("count", 0, 1).is_err());
        assert!(check::at_least("boost", 1.5f32, 1.0f32).is_ok());
    }
}
