use std::time::Duration;

use crate::{config::settings::get_user_config, web::api::get_api_url};
use devalang_types::{TelemetryEvent, TelemetrySendError};

pub async fn send_telemetry_event(event: &TelemetryEvent) -> Result<(), TelemetrySendError> {
    if let Some(cfg) = get_user_config() {
        if cfg.telemetry.enabled == false {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    let telemetry_url = format!("{}/v1/telemetry/send", get_api_url());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| TelemetrySendError::Http(format!("client build error: {}", e)))?;

    let mut last_err: Option<String> = None;
    for (i, delay_ms) in [0u64, 250, 500, 1000].iter().enumerate() {
        if *delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
        }

        let res = client
            .post(telemetry_url.clone())
            .json(event)
            .send()
            .await
            .and_then(|r| r.error_for_status());

        match res {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                last_err = Some(err.to_string());

                if i == 3 {
                    break;
                }
            }
        }
    }

    Err(TelemetrySendError::Http(
        last_err.unwrap_or_else(|| "unknown error".to_string()),
    ))
}
