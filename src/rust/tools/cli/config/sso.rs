/// Gets the SSO URL from environment or default
pub fn get_sso_url() -> String {
    std::env::var("DEVALANG_SSO_URL").unwrap_or_else(|_| "https://sso.devalang.com".to_string())
}
