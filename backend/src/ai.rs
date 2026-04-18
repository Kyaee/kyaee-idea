use std::env;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::error::AppError;
use crate::state::AppState;

/// Returns true if the claim should be treated as VERIFIED.
pub async fn evaluate_claim(
    state: &AppState,
    image: &[u8],
    content_type_hint: Option<&str>,
) -> Result<bool, AppError> {
    if let Ok(mock) = env::var("MOCK_AI_STATUS") {
        let m = mock.to_ascii_uppercase();
        if m == "REJECTED" || m == "FAIL" {
            return Ok(false);
        }
        if m == "VERIFIED" || m == "OK" {
            return Ok(true);
        }
    }

    let key = match env::var("OPENAI_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        Ok(_) | Err(_) => {
            tracing::info!("OPENAI_API_KEY unset; defaulting AI to VERIFIED (dev mode)");
            return Ok(true);
        }
    };

    let b64 = STANDARD.encode(image);
    let mime = content_type_hint
        .unwrap_or("image/jpeg")
        .to_string()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '/' || *c == '-' || *c == '+')
        .collect::<String>();

    let url = "https://api.openai.com/v1/chat/completions";
    let body = serde_json::json!({
        "model": env::var("OPENAI_VISION_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text": "You verify river cleanup bounty photos. Reply with exactly one word: VERIFIED if the image clearly shows a filled trash bag or collected waste suitable for a cleanup bounty; otherwise REJECTED."
                },
                {
                    "type": "image_url",
                    "image_url": { "url": format!("data:{mime};base64,{b64}") }
                }
            ]
        }],
        "max_tokens": 8
    });

    let res = state
        .http
        .post(url)
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("openai request: {e}"),
        })?;

    if !res.status().is_success() {
        let txt = res.text().await.unwrap_or_default();
        tracing::warn!("OpenAI error body: {txt}");
        return Err(AppError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: "openai returned error".into(),
        });
    }

    let v: serde_json::Value = res.json().await.map_err(|e| AppError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        message: format!("openai json: {e}"),
    })?;

    let text = v["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_ascii_uppercase();

    Ok(text.contains("VERIFIED") && !text.contains("REJECTED"))
}
