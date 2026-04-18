use std::env;
use std::process::Stdio;

use serde::Serialize;

use crate::error::AppError;

const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

#[derive(Clone, Serialize)]
pub struct DisbursementOutcome {
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl DisbursementOutcome {
    pub fn skipped(reason: &'static str) -> Self {
        Self {
            mode: "skipped",
            tx_hash: None,
            detail: Some(reason.into()),
        }
    }

    pub fn simulated(note: impl Into<String>) -> Self {
        Self {
            mode: "simulated",
            tx_hash: None,
            detail: Some(note.into()),
        }
    }

    pub fn live(tx_hash: String) -> Self {
        Self {
            mode: "live",
            tx_hash: Some(tx_hash),
            detail: None,
        }
    }
}

/// Minimal Stellar G-account check (StrKey); does not validate checksum on-chain.
pub fn validate_collector_address(addr: &str) -> Result<(), AppError> {
    let t = addr.trim();
    if t.len() != 56 {
        return Err(AppError::bad_request("collector address must be 56 characters"));
    }
    if !t.starts_with('G') {
        return Err(AppError::bad_request("collector address must start with G"));
    }
    if !t.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AppError::bad_request("invalid characters in collector address"));
    }
    Ok(())
}

pub async fn maybe_disburse(collector: &str) -> DisbursementOutcome {
    let use_cli = env::var("USE_STELLAR_CLI")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let contract = match env::var("RIVER_WARRIOR_CONTRACT_ID") {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            return DisbursementOutcome::simulated(
                "set RIVER_WARRIOR_CONTRACT_ID + USE_STELLAR_CLI=1 + STELLAR_ADMIN_SECRET for live invoke",
            );
        }
    };

    let admin_secret = match env::var("STELLAR_ADMIN_SECRET") {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            return DisbursementOutcome::simulated(
                "STELLAR_ADMIN_SECRET not set; skipping on-chain invoke",
            );
        }
    };

    if !use_cli {
        return DisbursementOutcome::simulated(
            "USE_STELLAR_CLI not enabled; no subprocess invoke (safe default)",
        );
    }

    let rpc = env::var("STELLAR_RPC_URL").unwrap_or_else(|_| {
        "https://soroban-testnet.stellar.org".to_string()
    });
    let passphrase =
        env::var("STELLAR_NETWORK_PASSPHRASE").unwrap_or_else(|_| TESTNET_PASSPHRASE.to_string());

    let bin = env::var("STELLAR_CLI_BIN").unwrap_or_else(|_| "stellar".to_string());

    let mut cmd = tokio::process::Command::new(&bin);
    cmd.arg("contract")
        .arg("invoke")
        .arg("--rpc-url")
        .arg(&rpc)
        .arg("--network-passphrase")
        .arg(&passphrase)
        .arg("--source")
        .arg(&admin_secret)
        .arg("--id")
        .arg(&contract)
        .arg("--")
        .arg("disburse_reward")
        .arg("--collector")
        .arg(collector.trim())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = match cmd.output().await {
        Ok(o) => o,
        Err(e) => {
            tracing::error!("stellar cli spawn failed: {e}");
            return DisbursementOutcome {
                mode: "error",
                tx_hash: None,
                detail: Some(format!("cli spawn: {e}")),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if !out.status.success() {
        tracing::warn!("stellar cli stderr: {stderr}");
        return DisbursementOutcome {
            mode: "error",
            tx_hash: None,
            detail: Some(format!("cli failed: {stderr}")),
        };
    }

    let hash = parse_tx_hash(&stdout).or_else(|| parse_tx_hash(&stderr));
    match hash {
        Some(h) => DisbursementOutcome::live(h),
        None => DisbursementOutcome {
            mode: "live",
            tx_hash: None,
            detail: Some(format!("invoke ok; parse tx hash from output: {stdout}")),
        },
    }
}

fn parse_tx_hash(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.len() == 64 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(line.to_lowercase());
        }
        if let Some(idx) = line.find("Transaction hash:") {
            let rest = line[idx + "Transaction hash:".len()..].trim();
            if rest.len() == 64 {
                return Some(rest.to_lowercase());
            }
        }
    }
    None
}
