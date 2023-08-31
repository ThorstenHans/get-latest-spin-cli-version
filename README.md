# get-latest-spin-cli-version

This repository contains a Fermyon Spin application, that exposes information about the latest `spin` CLI version. To do so, a GitHub Personal Access Token (PAT) is used. The spin application picks it up from the environment variable called `github_token`.

If `github_token` is not provided, the app will respond to all requests with an HTTP status code `500`.

## Available Endpoints

- GET `/version` will always return the version of the latest stable release as `text/plain` (e.G. `1.4.1`)
- GET `/version/stable` will return release information of the latest **stable** release as `application/json` (see sample response below)
- GET `/version/canary` will return release information of the latest **canary** release as `application/json` (see sample response below)

### Example Release Information

Sample response for `/version/stable`

```json
{
    "version": "1.4.2",
    "commit_hash": "main",
    "published_at": "2023-08-30T22:27:45Z"
}
```

Sample response for `/version/canary`

```json
{
    "version": "canary",
    "commit_hash": "16dd1339930123e6e9f1f9ec77bf058b72695afc",
    "published_at": "2023-08-29T07:55:40Z"
}
```
