//! Type validation utilities

use crate::error::{Result, ValidationWarning};
use super::L1Properties;

/// Validate L1 properties for consistency
pub fn validate_l1_properties(props: &L1Properties) -> Result<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();

    // Check for missing recommended fields
    if props.title.is_none() {
        warnings.push(ValidationWarning {
            message: "Document has no title".to_string(),
            field: Some("title".to_string()),
            position: None,
            suggestion: Some("Add a 'title' field to frontmatter".to_string()),
        });
    }

    // Check refinement vs stub count consistency
    if props.refinement.value() > 0.9 && !props.stubs.is_empty() {
        warnings.push(ValidationWarning {
            message: format!(
                "High refinement ({:.2}) but {} stubs remain",
                props.refinement.value(),
                props.stubs.len()
            ),
            field: Some("refinement".to_string()),
            position: None,
            suggestion: Some("Consider resolving stubs or lowering refinement".to_string()),
        });
    }

    // Check for blocking stubs with high refinement
    if props.refinement.value() > 0.7 && props.has_blocking_stubs() {
        warnings.push(ValidationWarning {
            message: "Document has blocking stubs but refinement > 0.7".to_string(),
            field: Some("stubs".to_string()),
            position: None,
            suggestion: Some("Blocking stubs should be resolved before high refinement".to_string()),
        });
    }

    Ok(warnings)
}

/// Validate that refinement meets audience gate
pub fn validate_audience_fit(props: &L1Properties) -> bool {
    props.audience.meets_gate(props.refinement.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Refinement;
    use crate::stubs::Stub;

    #[test]
    fn test_validates_missing_title() {
        let props = L1Properties::default();
        let warnings = validate_l1_properties(&props).unwrap();
        assert!(warnings.iter().any(|w| w.field.as_deref() == Some("title")));
    }

    #[test]
    fn test_validates_high_refinement_with_stubs() {
        let mut props = L1Properties::default();
        props.refinement = Refinement::new_unchecked(0.95);
        props.title = Some("Test".to_string());
        props.stubs = vec![Stub::compact("link", "Citation needed")];

        let warnings = validate_l1_properties(&props).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("stubs remain")));
    }
}
