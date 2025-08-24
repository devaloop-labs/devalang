use std::fs;
use tiny_http::{ Server, Response };
use webbrowser;
use serde::{ Serialize, Deserialize };
use dirs::home_dir;
use crate::common::sso::get_sso_url;
use std::{ thread, time::Duration };
use tiny_http::Header;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
struct UserConfig {
    token: String,
}

/// Handle the login command
/// This function initiates the login process by opening the browser and waiting for the callback.
#[cfg(feature = "cli")]
pub async fn handle_login_command() -> Result<(), String> {
    use crate::utils::spinner::with_spinner;
    use crate::utils::logger::Logger;
    use crate::utils::logger::LogLevel;

    let logger = Logger::new();

    let mut listener_port = 7878;

    let test_port_already_in_use = format!("127.0.0.1:{}", listener_port);
    while std::net::TcpListener::bind(&test_port_already_in_use).is_err() {
        listener_port += 1;
    }

    let redirect_uri = format!("http://127.0.0.1:{}/callback", listener_port);
    let login_url = format!(
        "{}/?response_type=code&referer=cli&redirect_uri={}",
        get_sso_url(),
        redirect_uri
    );

    if webbrowser::open(&login_url).is_ok() {
        logger.log_message(LogLevel::Info, "Opening browser for login...");
        logger.log_message(
            LogLevel::Info,
            &format!("If the browser does not open, please visit the following URL: {}", login_url)
        );
    } else {
        logger.log_message(
            LogLevel::Info,
            "Please open the following URL in your browser to login:"
        );
        logger.log_message(LogLevel::Info, &login_url);
    }

    let server = Server::http(format!("127.0.0.1:{}", listener_port)).unwrap();

    let spinner = with_spinner("Waiting for authentication...", || {
        thread::sleep(Duration::from_millis(800));
    });

    for request in server.incoming_requests() {
        let query = request.url().to_string();
        if request.url().starts_with("/callback") {
            if query.contains("session=") || query.contains("error=") {
                let token = query.split("session=").nth(1).unwrap_or("").to_string();

                if token.len() > 0 {
                    let response_html =
                        r#"
                        <html>
                            <body>
                                <h1>Authentication Successful</h1>
                                <h2>You can now close this window.</h2>
                            </body>
                        </html>
                    "#;

                    let response = Response::from_string(response_html)
                        .with_status_code(200)
                        .with_header(Header::from_str("Content-Type: text/html").unwrap());

                    request.respond(response).unwrap();

                    save_token(&token);

                    spinner.finish_and_clear();

                    logger.log_message(
                        LogLevel::Success,
                        "Authentication successful. Token saved to ~/.devalang/session_token.json"
                    );

                    break;
                } else {
                    spinner.finish_and_clear();
                    logger.log_message(LogLevel::Error, "Invalid session token.");
                    request.respond(Response::from_string("Invalid session token.")).unwrap();

                    break;
                }
            } else {
                println!("Invalid callback: {}", request.url());

                spinner.finish_and_clear();
                logger.log_message(LogLevel::Error, "Invalid callback.");
                request.respond(Response::from_string("Invalid callback.")).unwrap();

                break;
            }
        } else if request.url().starts_with("/favicon.ico") {
            // Ignore favicon requests
        } else {
            spinner.finish_and_clear();
            logger.log_message(LogLevel::Error, "Invalid request.");
            request.respond(Response::from_string("Invalid request.")).unwrap();

            break;
        }
    }

    Ok(())
}

/// Save the session token to a file in the user's home directory
fn save_token(token: &str) {
    let config = UserConfig { token: token.to_string() };
    let json = serde_json::to_string(&config).unwrap();

    let mut path = home_dir().unwrap();
    path.push(".devalang");
    fs::create_dir_all(&path).unwrap();

    path.push("session_token.json");
    fs::write(path, json).unwrap();
}
