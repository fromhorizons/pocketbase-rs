use std::fmt::Display;

enum EmailTemplate {
    Verification,
    PasswordReset,
    EmailChange,
}

impl Display for EmailTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verification => write!(f, "verification"),
            Self::PasswordReset => write!(f, "password-reset"),
            Self::EmailChange => write!(f, "email-change"),
        }
    }
}
