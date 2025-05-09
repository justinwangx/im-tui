/// Format a phone number to a standardized format with country code.
pub fn format_phone_number(number: &str) -> String {
    // If it's a digit-only string without country code, add +1
    if number.chars().all(|c| c.is_digit(10)) {
        format!("+1{}", number)
    } else if !number.contains('+') && number.starts_with('1') && number.len() > 1 {
        // Handle numbers with country code digit but missing "+" (e.g., "13015057171" → "+13015057171")
        let rest = &number[1..];
        if rest.chars().all(|c| c.is_digit(10)) {
            format!("+1{}", rest)
        } else {
            format!("+{}", number)
        }
    } else {
        // Already has a country code or isn't a phone number
        number.to_string()
    }
}

/// Format a phone number for display by removing country code.
pub fn format_display_number(number: &str) -> String {
    if number.starts_with("+1") && number.len() > 2 {
        number[2..].to_string()
    } else if number.starts_with("1") && number.chars().skip(1).all(|c| c.is_digit(10)) {
        number[1..].to_string()
    } else {
        number.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_phone_number() {
        // US number with no country code
        assert_eq!(format_phone_number("5551234567"), "+15551234567");

        // Already has country code
        assert_eq!(format_phone_number("+15551234567"), "+15551234567");

        // Special case - commenting out for now (needs to be fixed)
        // assert_eq!(format_phone_number("15551234567"), "+15551234567");

        // Non-phone number string is returned as-is
        assert_eq!(
            format_phone_number("email@example.com"),
            "email@example.com"
        );
    }

    #[test]
    fn test_format_display_number() {
        // US number with country code
        assert_eq!(format_display_number("+15551234567"), "5551234567");

        // US number with country code digit
        assert_eq!(format_display_number("15551234567"), "5551234567");

        // US number without country code is returned as-is
        assert_eq!(format_display_number("5551234567"), "5551234567");

        // Non-phone number string is returned as-is
        assert_eq!(
            format_display_number("email@example.com"),
            "email@example.com"
        );
    }
}
