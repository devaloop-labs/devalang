pub fn get_sso_url() -> String {
    let sso_url = std::env
        ::var("SSO_URL")
        .unwrap_or_else(|_| "https://sso.devalang.com".to_string());
        // .unwrap_or_else(|_| "http://localhost:5174".to_string());

    sso_url
}
