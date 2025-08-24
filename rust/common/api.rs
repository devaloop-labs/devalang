pub fn get_api_url() -> String {
    let api_url = std::env
        ::var("API_URL")
        .unwrap_or_else(|_| "https://api.devalang.com".to_string());
        // .unwrap_or_else(|_| "http://127.0.0.1:8989".to_string());

    api_url
}