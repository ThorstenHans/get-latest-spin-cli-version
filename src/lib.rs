#![warn(rust_2018_idioms)]

use anyhow::{bail, Result};

use serde::Serialize;
use spin_sdk::{
    http::{
        send, IntoResponse, Params, Request, RequestBuilder, Response, ResponseBuilder, Router,
    },
    http_component, variables,
};

/// A simple Spin HTTP component.
#[http_component]
fn handle_check_spin_cli_version(req: Request) -> Result<impl IntoResponse> {
    let mut router = Router::default();
    router.get_async("/version/:channel", get_version_by_channel);
    router.get_async("/version", get_latest_stable_version);
    Ok(router.handle(req))
}

#[derive(Debug, Serialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub commit_hash: String,
    pub published_at: String,
}

impl TryFrom<&[u8]> for ReleaseInfo {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let json = serde_json::from_slice::<serde_json::Value>(value)?;
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
}

async fn get_latest_stable_version(_req: Request, _params: Params) -> Result<impl IntoResponse> {
    let ri = get_release_info(None).await?;

    Ok(ResponseBuilder::new(200)
        .header("Content-Type", "text/plain")
        .body(ri.version)
        .build())
}

async fn get_version_by_channel(_req: Request, params: Params) -> Result<impl IntoResponse> {
    let c = params.get("channel");
    let ri = get_release_info(c).await?;
    let body = serde_json::to_string(&ri)?;

    Ok(ResponseBuilder::new(200)
        .header("Content-Type", "application/json")
        .body(body)
        .build())
}

async fn get_release_info(channel: Option<&str>) -> Result<ReleaseInfo> {
    let url = match channel {
        Some("canary") => "https://api.github.com/repos/fermyon/spin/releases/tags/canary",
        _ => "https://api.github.com/repos/fermyon/spin/releases/latest",
    };
    let auth_header_value = get_authorization_header_value()?;

    let req = RequestBuilder::new(spin_sdk::http::Method::Get, url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "spin-cli-version-checker")
        .header("Authorization", auth_header_value)
        .body(())
        .build();

    let res: Response = send(req).await?;

    if res.status() != &200 {
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
    let token = variables::get("github_token")?;
    Ok(format!("token {}", token))
}
