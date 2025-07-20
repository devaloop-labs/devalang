pub fn get_cdn_url() -> String {
    let cdn_url = std::env
        ::var("CDN_URL")
        .unwrap_or_else(|_| "https://cdn.devalang.com".to_string());

    if !cdn_url.ends_with('/') {
        format!("{}/", cdn_url)
    } else {
        cdn_url
    }
}
