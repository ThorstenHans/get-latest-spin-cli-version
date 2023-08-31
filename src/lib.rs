#![warn(rust_2018_idioms)]

use anyhow::{bail, Result};

use serde::Serialize;
use spin_sdk::{
    config,
    http::{Params, Request, Response, Router},
    http_component, outbound_http,
};

/// A simple Spin HTTP component.
#[http_component]
fn handle_check_spin_cli_version(req: Request) -> Result<Response> {
    let mut router = Router::default();
    router.get("/version/:channel", get_version_by_channel);
    router.get("/version", get_latest_stable_version);
    router.handle(req)
}

#[derive(Debug, Serialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub commit_hash: String,
    pub published_at: String,
}

impl TryFrom<&Option<bytes::Bytes>> for ReleaseInfo {
    type Error = anyhow::Error;

    fn try_from(value: &Option<bytes::Bytes>) -> std::result::Result<Self, Self::Error> {
        match value {
            Some(body) => {
                let json = serde_json::from_slice::<serde_json::Value>(body)?;
                let mut version = json["tag_name"].as_str().unwrap_or("unknown");
                let commit_hash = json["target_commitish"].as_str().unwrap_or("unknown");
                let published_at = json["published_at"].as_str().unwrap_or("unknown");
                if version.starts_with('v') {
                    version = version[1..].trim();
                }
                Ok(ReleaseInfo {
                    version: version.to_string(),
                    commit_hash: commit_hash.to_string(),
                    published_at: published_at.to_string(),
                })
            }
            None => bail!("No body in response"),
        }
    }
}

fn get_latest_stable_version(_req: Request, _params: Params) -> Result<Response> {
    let ri = get_release_info(None)?;

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .body(Some(ri.version.into()))?)
}

fn get_version_by_channel(_req: Request, params: Params) -> Result<Response> {
    let c = params.get("channel");
    let ri = get_release_info(c)?;
    let body = serde_json::to_string(&ri)?;

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Some(body.into()))?)
}

fn get_release_info(channel: Option<&str>) -> Result<ReleaseInfo> {
    let url = match channel {
        Some("canary") => "https://api.github.com/repos/fermyon/spin/releases/tags/canary",
        _ => "https://api.github.com/repos/fermyon/spin/releases/latest",
    };
    let auth_header_value = get_authorization_header_value()?;

    let req = http::Request::builder()
        .method("GET")
        .uri(url)
        .header(http::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(http::header::USER_AGENT, "spin-cli-version-checker")
        .header(http::header::AUTHORIZATION, auth_header_value)
        .body(None)?;

    let Ok(res) = outbound_http::send_request(req) else {
        let err = "Could not determine latest Spin CLI release. (Could not send request)";
        println!("{}", err);
        bail!("{}", err)
    };
    if res.status() != http::StatusCode::OK {
        let err = format!(
            "Could not determine latest Spin CLI release. ({:?})",
            res.status()
        );
        println!("{}", err);
        bail!("{}", err)
    }
    ReleaseInfo::try_from(res.body())
}

fn get_authorization_header_value() -> Result<String> {
    let token = config::get("github_token")?;
    Ok(format!("token {}", token))
}
