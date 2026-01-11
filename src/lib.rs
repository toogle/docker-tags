use std::{cmp::Ordering, collections::HashMap, fmt, fs};

use anyhow::{Context, Result, anyhow};
use reqwest::{StatusCode, Url, header};
use semver::Version;
use serde::Deserialize;

/// Docker configuration
#[derive(Deserialize)]
struct DockerConfig {
    auths: HashMap<String, DockerAuth>,
}

#[derive(Deserialize)]
struct DockerAuth {
    auth: String,
}

/// Structure for authentication response
#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

/// Structure for tags response
#[derive(Deserialize)]
struct TagsResponse {
    tags: Vec<String>,
}

/// A Docker image representation
#[derive(Debug)]
pub struct Image {
    registry: String,
    repository: String,
}

impl Image {
    pub fn new(registry: impl Into<String>, repository: impl Into<String>) -> Self {
        Image {
            registry: registry.into(),
            repository: repository.into(),
        }
    }

    fn read_auth_token(&self) -> Result<Option<String>> {
        let path = shellexpand::tilde("~/.docker/config.json").to_string();
        if let Ok(contents) = fs::read_to_string(path) {
            let config: DockerConfig =
                serde_json::from_str(&contents).context("Failed to parse Docker config")?;
            let registry = match self.registry.as_str() {
                "docker.io" => "https://index.docker.io/v1/",
                registry => registry,
            };
            return Ok(config.auths.get(registry).map(|a| a.auth.clone()));
        }

        Ok(None)
    }

    async fn handle_auth_challenge(&self, hdr: &str) -> Result<(String, String)> {
        let (scheme, rest) = hdr
            .split_once(' ')
            .ok_or(anyhow!("Invalid authentication header: {hdr}"))?;
        let mut params = HashMap::new();
        for param in rest.split(',') {
            if let Some((k, v)) = param.split_once('=') {
                params.insert(k.trim(), v.trim().trim_matches('"'));
            }
        }

        let realm = params
            .remove("realm")
            .with_context(|| format!("No realm found in WWW-Authenticate header: {hdr}"))?;
        let url = Url::parse_with_params(realm, params)
            .with_context(|| format!("Failed to parse realm URL: {realm}"))?;

        let mut req = reqwest::Client::new().get(url.clone());
        if let Some(auth_token) = self.read_auth_token()? {
            req = req.header(header::AUTHORIZATION, format!("Basic {auth_token}"));
        }

        let resp = req
            .send()
            .await
            .with_context(|| format!("Failed to fetch token from {url}"))?;
        let data: TokenResponse = match resp.status() {
            StatusCode::OK => resp
                .json()
                .await
                .with_context(|| format!("Failed to parse token response from {url}"))?,
            status => return Err(anyhow!("Failed to authenticate: {status}")),
        };

        Ok((scheme.to_string(), data.token))
    }

    pub async fn fetch_tags(&self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();
        let client = reqwest::Client::new();
        let mut token = String::new();

        let registry = match self.registry.as_str() {
            "docker.io" => "registry-1.docker.io",
            registry => registry,
        };
        let repository = if self.registry == "docker.io" && !self.repository.contains('/') {
            &format!("library/{}", self.repository)
        } else {
            &self.repository
        };
        let url = format!("https://{}/v2/{}/tags/list?n=100", registry, repository,);
        let mut next_url = url.clone();
        loop {
            let mut req = client.get(&next_url);
            if !token.is_empty() {
                req = req.header(header::AUTHORIZATION, format!("Bearer {token}"));
            }

            let resp = req
                .send()
                .await
                .with_context(|| format!("Failed to fetch tags from {next_url:?}"))?;
            let data: TagsResponse = match resp.status() {
                StatusCode::OK => resp
                    .json()
                    .await
                    .with_context(|| format!("Failed to parse JSON from {next_url:?}"))?,
                StatusCode::UNAUTHORIZED
                    if resp.headers().contains_key(header::WWW_AUTHENTICATE) =>
                {
                    if !token.is_empty() {
                        return Err(anyhow!("Got HTTP 401 with authentication token")
                            .context("Image not found"));
                    }

                    let hdr = resp
                        .headers()
                        .get(header::WWW_AUTHENTICATE)
                        .unwrap()
                        .to_str()
                        .context("Failed to parse WWW-Authenticate header")?;
                    (_, token) = self
                        .handle_auth_challenge(hdr)
                        .await
                        .context("Image not found")?;
                    continue;
                }
                StatusCode::NOT_FOUND => return Err(anyhow!("Image not found")),
                status => return Err(anyhow!(status)),
            };

            let page_tags: Vec<_> = data.tags.into_iter().map(|tag| Tag { name: tag }).collect();
            let page_len = page_tags.len();
            let last_tag = page_tags[page_len - 1].name.clone();
            tags.extend(page_tags);

            if page_len < 100 {
                break;
            } else {
                next_url = Url::parse_with_params(&url, &[("last", last_tag)])
                    .with_context(|| format!("Failed to parse URL: {next_url:?}"))?
                    .to_string();
            }
        }

        Ok(tags)
    }
}

impl TryFrom<&str> for Image {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split("/").collect();
        match parts.len() {
            1 => Ok(Image::new("docker.io", value)),
            2 => match parts[0].contains('.') {
                true => Ok(Image::new(parts[0], parts[1])),
                false => Ok(Image::new("docker.io", value)),
            },
            3 if parts[0].contains('.') => {
                Ok(Image::new(parts[0], format!("{}/{}", parts[1], parts[2])))
            }
            _ => Err("Invalid image format"),
        }
    }
}

/// A Docker image tag representation
#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    name: String,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = Version::parse(self.name.trim_start_matches('v'));
        let b = Version::parse(other.name.trim_start_matches('v'));
        match (a, b) {
            (Ok(a), Ok(b)) => b.cmp(&a),       // latest versions first
            (Ok(_), Err(_)) => Ordering::Less, // alphanumeric tags at the end
            (Err(_), Ok(_)) => Ordering::Greater,
            _ => self.name.cmp(&other.name), // and sorted alphabetically
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_try_from() {
        assert!(matches!(
            Image::try_from("debian"),
            Ok(Image { registry, repository })
                if registry == "docker.io" && repository == "debian"
        ));

        assert!(matches!(
            Image::try_from("prom/prometheus"),
            Ok(Image { registry, repository })
                if registry == "docker.io" && repository == "prom/prometheus"
        ));

        assert!(matches!(
            Image::try_from("docker.angie.software/angie"),
            Ok(Image { registry, repository })
                if registry == "docker.angie.software" && repository == "angie"
        ));

        assert!(matches!(
            Image::try_from("docker.io/prom/prometheus"),
            Ok(Image { registry, repository })
                if registry == "docker.io" && repository == "prom/prometheus"
        ));

        assert!(matches!(
            Image::try_from("quay.io/prometheus/prometheus"),
            Ok(Image { registry, repository })
                if registry == "quay.io" && repository == "prometheus/prometheus"
        ));
    }

    #[test]
    fn test_image_try_from_invalid() {
        assert!(matches!(
            Image::try_from("invalid/image/format"),
            Err("Invalid image format")
        ));

        assert!(matches!(
            Image::try_from("another.com/invalid/image/format"),
            Err("Invalid image format")
        ));
    }
}
