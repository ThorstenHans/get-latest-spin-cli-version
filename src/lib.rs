use anyhow::{Result, bail};

use spin_sdk::{
    http::{Request, Response, Router, Params},
    http_component, outbound_http, config,
};

fn get_authorization_header_value() -> Result<String> {
    let token = config::get("github_token")?;
    Ok(format!("token {}", token))
}

fn handle_get_latest_spin_cli_version(_req: Request, _params : Params) -> Result<Response> {
    let auth_header_value = get_authorization_header_value()?;
    let req = http::Request::builder()
        .method("GET")
        .uri("https://api.github.com/repos/fermyon/spin/releases/latest")
        .header(http::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(http::header::USER_AGENT, "spin-cli-version-checker")
        .header(http::header::AUTHORIZATION, auth_header_value)
        .body(None)?;

    let Ok(res) = outbound_http::send_request(req) else {
        let err = "Could not determine latest Spin CLI release. (Could not send request)";
        println!("{}", err);
        return Ok(http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Some(err.into()))?);
    };
    if res.status() != http::StatusCode::OK {
        let err = format!("Could not determine latest Spin CLI release. ({:?})", res.status());
        println!("{}", err);
        return Ok(http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Some(err.into()))?);
    }
    let version = extract_version(res.body())?;
    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .body(Some(version.into()))?)
    
}

fn extract_version(body: &Option<bytes::Bytes>) -> Result<String> {
    match body {
        Some(body) => {
            let json = serde_json::from_slice::<serde_json::Value>(body)?;
            let version = json["tag_name"].as_str().unwrap_or("unknown");
            Ok(version.to_string())
        },
        None => bail!("No body in response")
    }
}
/// A simple Spin HTTP component.
#[http_component]
fn handle_check_spin_cli_version(req: Request) -> Result<Response> {
    let mut router = Router::default();
    router.get("/version", handle_get_latest_spin_cli_version);
    
    router.handle(req)
}
