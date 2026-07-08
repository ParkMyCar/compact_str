use compact_str::CompactString;
use garde::Validate;

/// A user registration form with CompactString fields validated by garde.
///
/// CompactString implements all of garde's string traits, so it works as a
/// drop-in replacement for String in validated structs.
#[derive(Debug, Validate)]
struct RegistrationForm {
    /// 3–32 chars
    #[garde(length(chars, min = 3, max = 32))]
    username: CompactString,

    /// At least 8 bytes (handles multi-byte passwords correctly)
    #[garde(length(bytes, min = 8))]
    password: CompactString,

    /// Must be ASCII only
    #[garde(ascii, length(min = 1, max = 64))]
    invite_code: CompactString,
}

fn main() {
    let valid = RegistrationForm {
        username: CompactString::from("alice"),
        password: CompactString::from("s3cur3p@ss"),
        invite_code: CompactString::from("WELCOME2024"),
    };
    match valid.validate_with(&()) {
        Ok(()) => println!("Valid form: {:?}", valid),
        Err(e) => println!("Unexpected error: {e}"),
    }

    let short_username = RegistrationForm {
        username: CompactString::from("al"),
        password: CompactString::from("s3cur3p@ss"),
        invite_code: CompactString::from("WELCOME2024"),
    };
    match short_username.validate() {
        Ok(()) => println!("Unexpected success"),
        Err(e) => println!("Validation error (username too short):\n{e}"),
    }

    let short_password = RegistrationForm {
        username: CompactString::from("bob"),
        password: CompactString::from("short"),
        invite_code: CompactString::from("WELCOME2024"),
    };
    match short_password.validate() {
        Ok(()) => println!("Unexpected success"),
        Err(e) => println!("Validation error (password too short):\n{e}"),
    }

    let non_ascii_code = RegistrationForm {
        username: CompactString::from("carol"),
        password: CompactString::from("s3cur3p@ss"),
        invite_code: CompactString::from("WËLCOME"),
    };
    match non_ascii_code.validate() {
        Ok(()) => println!("Unexpected success"),
        Err(e) => println!("Validation error (non-ASCII invite code):\n{e}"),
    }
}
