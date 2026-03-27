use validator::ValidationError;

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    // Simple phone validation - can be enhanced
    if phone.len() < 10 {
        return Err(ValidationError::new("phone_too_short"));
    }
    Ok(())
}
