use crate::column::CellValidator;

// ── validation rule ──────────────────────────────────────

/// A single declarative business-rule check for a column's cell
/// values.
///
/// Rules are evaluated in order on [`ColumnDef::rules`]
/// (crate::column::ColumnDef); the first failure wins. Each
/// built-in variant carries a default error message that can be
/// overridden with `.with_message(...)`. For anything not covered
/// by the built-ins (regex-like patterns, cross-field checks...),
/// use [`ValidationRule::Custom`] with a [`CellValidator`] closure.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// The value must not be empty (after trimming whitespace).
    Required {
        /// Error message shown on failure.
        message: Option<String>,
    },
    /// The value must be at least `min` characters long.
    MinLength {
        /// Minimum accepted length, in characters.
        min: usize,
        /// Error message shown on failure.
        message: Option<String>,
    },
    /// The value must be at most `max` characters long.
    MaxLength {
        /// Maximum accepted length, in characters.
        max: usize,
        /// Error message shown on failure.
        message: Option<String>,
    },
    /// The value, parsed as `f64`, must fall within `min..=max`.
    /// Values that fail to parse as a number are rejected.
    Range {
        /// Minimum accepted value (inclusive).
        min: f64,
        /// Maximum accepted value (inclusive).
        max: f64,
        /// Error message shown on failure.
        message: Option<String>,
    },
    /// The value must exactly match one entry in `values`
    /// (an allowed-value list).
    OneOf {
        /// Allowed values.
        values: Vec<String>,
        /// Error message shown on failure.
        message: Option<String>,
    },
    /// Arbitrary validation logic, reusing the existing
    /// [`CellValidator`] closure type.
    Custom(CellValidator),
}

impl ValidationRule {
    /// The value must not be empty (after trimming whitespace).
    pub fn required() -> Self {
        Self::Required { message: None }
    }

    /// The value must be at least `min` characters long.
    pub fn min_length(min: usize) -> Self {
        Self::MinLength { min, message: None }
    }

    /// The value must be at most `max` characters long.
    pub fn max_length(max: usize) -> Self {
        Self::MaxLength { max, message: None }
    }

    /// The value, parsed as `f64`, must fall within `min..=max`.
    pub fn range(min: f64, max: f64) -> Self {
        Self::Range {
            min,
            max,
            message: None,
        }
    }

    /// The value must exactly match one entry in `values`.
    pub fn one_of(values: Vec<String>) -> Self {
        Self::OneOf {
            values,
            message: None,
        }
    }

    /// Override the default error message of a built-in rule.
    /// No-op on [`ValidationRule::Custom`], which carries its own
    /// message via the wrapped closure's `Err(String)`.
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        let msg = msg.into();
        match &mut self {
            Self::Required { message }
            | Self::MinLength { message, .. }
            | Self::MaxLength { message, .. }
            | Self::Range { message, .. }
            | Self::OneOf { message, .. } => *message = Some(msg),
            Self::Custom(_) => {}
        }
        self
    }

    /// Evaluate this rule against a raw cell value.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            Self::Required { message } => {
                if value.trim().is_empty() {
                    Err(message
                        .clone()
                        .unwrap_or_else(|| "This field is required.".into()))
                } else {
                    Ok(())
                }
            }
            Self::MinLength { min, message } => {
                if value.chars().count() < *min {
                    Err(message.clone().unwrap_or_else(|| {
                        format!("Must be at least {min} characters long.")
                    }))
                } else {
                    Ok(())
                }
            }
            Self::MaxLength { max, message } => {
                if value.chars().count() > *max {
                    Err(message.clone().unwrap_or_else(|| {
                        format!("Must be at most {max} characters long.")
                    }))
                } else {
                    Ok(())
                }
            }
            Self::Range { min, max, message } => match value.parse::<f64>() {
                Ok(v) if v >= *min && v <= *max => Ok(()),
                _ => Err(message.clone().unwrap_or_else(|| {
                    format!("Must be a number between {min} and {max}.")
                })),
            },
            Self::OneOf { values, message } => {
                if values.iter().any(|v| v == value) {
                    Ok(())
                } else {
                    Err(message.clone().unwrap_or_else(|| {
                        format!("Must be one of: {}.", values.join(", "))
                    }))
                }
            }
            Self::Custom(validator) => validator.validate(value),
        }
    }
}

/// Evaluate a slice of rules against `value`, returning the first
/// failure (rules run in order).
pub(crate) fn validate_rules(
    rules: &[ValidationRule],
    value: &str,
) -> Result<(), String> {
    for rule in rules {
        rule.validate(value)?;
    }
    Ok(())
}

// ── invalid-edit mode ────────────────────────────────────

/// Grid-wide policy applied when a `CommitEdit` fails validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum InvalidEditMode {
    /// Reject the edit and revert the cell to its previous value.
    /// The editor closes as if the user had pressed Escape.
    #[default]
    Revert,
    /// Keep the editor open (and the edit session active) until the
    /// value passes validation or the user explicitly cancels.
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::CellValidator;

    #[test]
    fn required_rejects_empty_and_whitespace() {
        let r = ValidationRule::required();
        assert!(r.validate("").is_err());
        assert!(r.validate("   ").is_err());
        assert!(r.validate("x").is_ok());
    }

    #[test]
    fn min_length_rejects_short_values() {
        let r = ValidationRule::min_length(3);
        assert!(r.validate("ab").is_err());
        assert!(r.validate("abc").is_ok());
    }

    #[test]
    fn max_length_rejects_long_values() {
        let r = ValidationRule::max_length(3);
        assert!(r.validate("abcd").is_err());
        assert!(r.validate("abc").is_ok());
    }

    #[test]
    fn range_rejects_out_of_bounds_and_non_numeric() {
        let r = ValidationRule::range(0.0, 100.0);
        assert!(r.validate("50").is_ok());
        assert!(r.validate("0").is_ok());
        assert!(r.validate("100").is_ok());
        assert!(r.validate("-1").is_err());
        assert!(r.validate("101").is_err());
        assert!(r.validate("abc").is_err());
    }

    #[test]
    fn one_of_rejects_values_outside_the_list() {
        let r = ValidationRule::one_of(vec!["A".into(), "B".into()]);
        assert!(r.validate("A").is_ok());
        assert!(r.validate("C").is_err());
    }

    #[test]
    fn custom_delegates_to_cell_validator() {
        let r = ValidationRule::Custom(CellValidator::new(|v| {
            if v == "ok" {
                Ok(())
            } else {
                Err("nope".into())
            }
        }));
        assert!(r.validate("ok").is_ok());
        assert!(r.validate("bad").is_err());
    }

    #[test]
    fn with_message_overrides_default() {
        let r = ValidationRule::required().with_message("custom message");
        assert_eq!(r.validate("").unwrap_err(), "custom message");
    }

    #[test]
    fn with_message_is_noop_on_custom() {
        let r = ValidationRule::Custom(CellValidator::new(|_| Err("x".into())))
            .with_message("ignored");
        assert_eq!(r.validate("").unwrap_err(), "x");
    }

    #[test]
    fn validate_rules_stops_at_first_failure() {
        let rules = vec![
            ValidationRule::required(),
            ValidationRule::min_length(5).with_message("too short"),
        ];
        assert_eq!(
            validate_rules(&rules, "").unwrap_err(),
            "This field is required."
        );
        assert_eq!(validate_rules(&rules, "ab").unwrap_err(), "too short");
        assert!(validate_rules(&rules, "abcdef").is_ok());
    }

    #[test]
    fn invalid_edit_mode_default_is_revert() {
        assert_eq!(InvalidEditMode::default(), InvalidEditMode::Revert);
    }
}
