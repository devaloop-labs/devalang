pub fn get_cdn_url() -> String {
    let cdn_url = std::env
        ::var("CDN_URL")
        .unwrap_or_else(|_| "https://cdn.devalang.com".to_string());
        // .unwrap_or_else(|_| "http://127.0.0.1:8888".to_string());

    cdn_url
}
