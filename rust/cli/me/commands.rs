pub async fn handle_me_command() -> Result<(), String> {
    let auth_url = crate::web::auth::get_auth_url();

    let url = format!("{}/v1/auth/me", auth_url);

    let client = reqwest::Client::new();
    let mut req = client.get(url);

    if let Some(user) = crate::config::settings::get_user_config() {
        if !user.session.is_empty() {
            req = req.bearer_auth(user.session);
        }
    } else {
        return Err("No user session found. Please log in.".to_string());
    }

    let response = req.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed to get user info: HTTP {}", response.status()).into());
    }

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let json: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Invalid JSON response (status {}): {}",
                status, body_text
            ));
        }
    };

    let payload = json.get("payload");

    let email = payload
        .unwrap_or(&serde_json::Value::Null)
        .get("userData")
        .unwrap_or(&serde_json::Value::Null)
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("Logged in as: {}", email);

    Ok(())
}
