# get-latest-spin-cli-version

This repository contains a spin application which will read the latest version (stable releases considered only) of `spin` CLI from GitHub. To do so, a GitHub Personal Access Token (PAT) is used. The spin application picks it up from the environment variable called `github_token`.

If `github_token` is not provided, the app will respond to all requests with an HTTP status code `500`.

The actual version of `spin` CLI is returned as HTTP response as `text/plain`
