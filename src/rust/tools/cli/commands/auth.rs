#![cfg(feature = "cli")]

use crate::tools::cli::config::sso::get_sso_url;
use crate::tools::cli::config::user::{clear_session_token, get_session_token, set_session_token};
use anyhow::Result;
use std::str::FromStr;
use tiny_http::{Header, Response, Server};

/// Login command with OAuth flow
/// Opens browser to SSO page and waits for callback with session token
pub async fn login(_token: Option<String>) -> Result<()> {
    use crate::tools::logger::Logger;
    let logger = Logger::new();
    logger.info("Starting authentication flow...");

    // Find an available port starting from 7878
    let mut listener_port = 7878;
    let test_port_already_in_use = format!("127.0.0.1:{}", listener_port);
    while std::net::TcpListener::bind(&test_port_already_in_use).is_err() {
        listener_port += 1;
    }

    // Build OAuth URLs
    let redirect_uri = format!("http://127.0.0.1:{}/callback", listener_port);
    let login_url = format!(
        "{}/?response_type=code&referer=cli&redirect_uri={}",
        get_sso_url(),
        redirect_uri
    );

    // Try to open browser
    logger.info("Opening browser for authentication...");
    if webbrowser::open(&login_url).is_ok() {
        logger.success("Browser opened successfully");
        logger.info(&format!(
            "If the browser didn't open, visit this URL:\n   {}",
            login_url
        ));
    } else {
        logger.error("Could not open browser automatically");
        logger.info(&format!(
            "Please open this URL in your browser:\n   {}",
            login_url
        ));
    }

    // Start local HTTP server to receive callback
    let server = Server::http(format!("127.0.0.1:{}", listener_port))
        .map_err(|e| anyhow::anyhow!("Failed to start local server: {}", e))?;

    logger.info(&format!(
        "Waiting for authentication... (Listening on http://127.0.0.1:{})",
        listener_port
    ));

    // Wait for callback
    for request in server.incoming_requests() {
        let query = request.url().to_string();

        // Handle callback with session token
        if request.url().starts_with("/callback") {
            if query.contains("session=") {
                // Extract token from query string
                let token = query
                    .split("session=")
                    .nth(1)
                    .and_then(|s| s.split('&').next())
                    .unwrap_or("")
                    .to_string();

                if !token.is_empty() {
                    // Send success response to browser
                    let response_html = r#"
                        <!DOCTYPE html>
                        <html>
                            <head>
                                <meta charset="UTF-8">
                                <title>Authentication Successful</title>
                                <style>
                                    body {
                                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                        display: flex;
                                        justify-content: center;
                                        align-items: center;
                                        height: 100vh;
                                        margin: 0;
                                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                                    }
                                    .container {
                                        text-align: center;
                                        background: white;
                                        padding: 3rem;
                                        border-radius: 1rem;
                                        box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                                    }
                                    h1 {
                                        color: #667eea;
                                        margin-bottom: 1rem;
                                        font-size: 2.5rem;
                                    }
                                    p {
                                        color: #666;
                                        font-size: 1.2rem;
                                    }
                                    .icon {
                                        font-size: 4rem;
                                        margin-bottom: 1rem;
                                    }
                                </style>
                            </head>
                            <body>
                                <div class="container">
                                    <div class="icon">✅</div>
                                    <h1>Authentication Successful!</h1>
                                    <p>You can now close this window and return to the terminal.</p>
                                </div>
                            </body>
                        </html>
                    "#;

                    let response = Response::from_string(response_html)
                        .with_status_code(200)
                        .with_header(
                            Header::from_str("Content-Type: text/html; charset=utf-8").unwrap(),
                        );

                    let _ = request.respond(response);

                    // Save token
                    set_session_token(token)?;

                    logger.success("Authentication successful!");
                    logger.info("Token saved to ~/.devalang/config.json");

                    return Ok(());
                } else {
                    let response_html = r#"
                        <!DOCTYPE html>
                        <html>
                            <head>
                                <meta charset="UTF-8">
                                <title>Authentication Failed</title>
                                <style>
                                    body {
                                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                        display: flex;
                                        justify-content: center;
                                        align-items: center;
                                        height: 100vh;
                                        margin: 0;
                                        background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                                    }
                                    .container {
                                        text-align: center;
                                        background: white;
                                        padding: 3rem;
                                        border-radius: 1rem;
                                        box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                                    }
                                    h1 {
                                        color: #f5576c;
                                        margin-bottom: 1rem;
                                        font-size: 2.5rem;
                                    }
                                    p {
                                        color: #666;
                                        font-size: 1.2rem;
                                    }
                                    .icon {
                                        font-size: 4rem;
                                        margin-bottom: 1rem;
                                    }
                                </style>
                            </head>
                            <body>
                                <div class="container">
                                    <div class="icon">❌</div>
                                    <h1>Authentication Failed</h1>
                                    <p>Invalid session token. Please try again.</p>
                                </div>
                            </body>
                        </html>
                    "#;

                    let response = Response::from_string(response_html)
                        .with_status_code(400)
                        .with_header(
                            Header::from_str("Content-Type: text/html; charset=utf-8").unwrap(),
                        );

                    let _ = request.respond(response);

                    return Err(anyhow::anyhow!("Invalid session token received from SSO"));
                }
            } else if query.contains("error=") {
                let error = query
                    .split("error=")
                    .nth(1)
                    .and_then(|s| s.split('&').next())
                    .unwrap_or("unknown")
                    .to_string();

                let response_html = format!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                        <head>
                            <meta charset="UTF-8">
                            <title>Authentication Error</title>
                            <style>
                                body {{
                                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    height: 100vh;
                                    margin: 0;
                                    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                                }}
                                .container {{
                                    text-align: center;
                                    background: white;
                                    padding: 3rem;
                                    border-radius: 1rem;
                                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                                }}
                                h1 {{
                                    color: #f5576c;
                                    margin-bottom: 1rem;
                                    font-size: 2.5rem;
                                }}
                                p {{
                                    color: #666;
                                    font-size: 1.2rem;
                                }}
                                .icon {{
                                    font-size: 4rem;
                                    margin-bottom: 1rem;
                                }}
                                .error {{
                                    background: #ffe0e0;
                                    padding: 1rem;
                                    border-radius: 0.5rem;
                                    margin-top: 1rem;
                                    color: #c00;
                                }}
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="icon">⚠️</div>
                                <h1>Authentication Error</h1>
                                <p>An error occurred during authentication.</p>
                                <div class="error">{}</div>
                            </div>
                        </body>
                    </html>
                "#,
                    error
                );

                let response = Response::from_string(response_html)
                    .with_status_code(400)
                    .with_header(
                        Header::from_str("Content-Type: text/html; charset=utf-8").unwrap(),
                    );

                let _ = request.respond(response);

                return Err(anyhow::anyhow!("Authentication error: {}", error));
            } else {
                let response_html = r#"
                    <!DOCTYPE html>
                    <html>
                        <head>
                            <meta charset="UTF-8">
                            <title>Invalid Callback</title>
                            <style>
                                body {
                                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    height: 100vh;
                                    margin: 0;
                                    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                                }
                                .container {
                                    text-align: center;
                                    background: white;
                                    padding: 3rem;
                                    border-radius: 1rem;
                                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                                }
                                h1 {
                                    color: #f5576c;
                                    margin-bottom: 1rem;
                                    font-size: 2.5rem;
                                }
                                p {
                                    color: #666;
                                    font-size: 1.2rem;
                                }
                                .icon {
                                    font-size: 4rem;
                                    margin-bottom: 1rem;
                                }
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="icon">❌</div>
                                <h1>Invalid Callback</h1>
                                <p>The authentication callback is invalid.</p>
                            </div>
                        </body>
                    </html>
                "#;

                let response = Response::from_string(response_html)
                    .with_status_code(400)
                    .with_header(
                        Header::from_str("Content-Type: text/html; charset=utf-8").unwrap(),
                    );

                let _ = request.respond(response);

                return Err(anyhow::anyhow!("Invalid callback received"));
            }
        }
        // Ignore favicon requests
        else if request.url().starts_with("/favicon.ico") {
            let response = Response::from_string("").with_status_code(404);
            let _ = request.respond(response);
        }
        // Handle any other invalid requests
        else {
            let response = Response::from_string("Invalid request").with_status_code(400);
            let _ = request.respond(response);
        }
    }

    Err(anyhow::anyhow!("Authentication flow interrupted"))
}

/// Logout command
pub async fn logout() -> Result<()> {
    use crate::tools::logger::Logger;
    let logger = Logger::new();

    if get_session_token().is_none() {
        logger.info("You are not logged in.");
        return Ok(());
    }

    clear_session_token()?;
    logger.success("Successfully logged out!");
    logger.info("Token removed from ~/.devalang/config.json");

    Ok(())
}

/// Command to check authentication status
pub async fn check_auth_status() -> Result<()> {
    use crate::tools::logger::Logger;
    let logger = Logger::new();

    if let Some(token) = get_session_token() {
        logger.success("You are logged in.");
        logger.info(&format!("Token: {}...", &token[..token.len().min(10)]));
    } else {
        logger.error("You are not logged in.");
        logger.info("Run 'devalang login' to authenticate.");
    }

    Ok(())
}
