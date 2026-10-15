//! Passwords are stored as bcrypt hashes.

/// bcrypt's cost: slow enough to make guessing expensive.
const COST: u32 = 12;

pub fn hash(password: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(password, COST)?)
}

/// Whether `password` matches the stored hash. A broken hash never matches.
pub fn verify(password: &str, digest: &str) -> bool {
    bcrypt::verify(password, digest).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_hashes_of_other_bcrypt_libraries() {
        // Made by Ruby's bcrypt gem (has_secure_password) for "heslo12345".
        let other = "$2a$04$e3UTqAmP.3xrk/k1K/dT8OnaE.WX2BB7y37fzLvGlm.kH4Ncu3PDy";
        assert!(verify("heslo12345", other));
        assert!(!verify("jine-heslo", other));
        assert!(!verify("cokoli", "not a hash"));
    }
}
