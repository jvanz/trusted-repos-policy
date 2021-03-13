use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chimera::settings::Trusties;

use anyhow::{anyhow, Result};

use core::fmt::Display;
use url::{Url, Host, ParseError};
use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Settings {
    registries: Registries,
    tags: Tags,
    images: Images,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Registries {
    allow: Vec<String>,
    reject: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Tags {
    allow: Vec<String>,
    reject: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Images {
    allow: Vec<String>,
    reject: Vec<String>,
}

impl Trusties for Settings {
    fn trusted_users(&self) -> HashSet<String> {
        Default::default()
    }

    fn trusted_groups(&self) -> HashSet<String> {
        Default::default()
    }
}

impl Settings {
    fn is_allowed_registry(&self, registry: String) -> bool {
        false
    }

    fn is_allowed_tag(&self, tag: String) -> bool {
        false
    }

    fn is_allowed_image(&self, image: String) -> bool {
        false
    }
}

#[derive(Default)]
struct Image {
    registry: Option<String>,
    fqn: String,
    name: String,
    tag: Option<String>,
    sha256: Option<String>,
}

impl Image {
    fn new<T>(image: T) -> Result<Image> where
        T: Into<String> + Display + Copy
    {
        println!("about to parse: '{}'", image);

        let image_with_scheme = format!("registry://{}", image);
        let url = Url::parse(&image_with_scheme);

        let registry = url.clone().and_then(|url| {
            url.host().map(|host| {
                match host {
                    Host::Domain(domain) => domain.into(),
                    Host::Ipv4(address) => format!("{}", address),
                    Host::Ipv6(address) => format!("{}", address),
                }
            }).ok_or(url::ParseError::EmptyHost)
        }).and_then(|host| {
            url.clone().map(|url| url.port().map_or(host.clone(), |port| format!("{}:{}", host, port)))
        });

        let parse_fqn = Regex::new(r"^(registry://)?(?P<fqn>[^:@]+)(:(?P<tag>[^@]+))?(@sha256:(?P<sha256>[A-Fa-f0-9]{64}))?$").unwrap();
        let parse_image_name = Regex::new(r"(?P<image>.*)$").unwrap();
        let parse_image_name_with_scheme = Regex::new(r"^registry://(?P<fqn>.*)$").unwrap();

        let parse_image_reference = if url.clone()?.path().is_empty() {
            &parse_image_name_with_scheme
        } else {
            &parse_fqn
        };

        parse_image_reference.captures(format!("{}", url?).as_ref()).map(|captures| {
            (
                captures.name("fqn").map(|fqn| fqn.as_str()),
                captures.name("tag").map(|tag| tag.as_str()),
                captures.name("sha256").map(|sha256| sha256.as_str()),
            )
        }).map(|(fqn, tag, sha256)| {
            Image {
                registry: registry.ok(),
                fqn: fqn.map_or(Default::default(), |fqn| fqn.to_string()),
                tag: tag.map(|tag| tag.to_string()),
                sha256: sha256.map(|sha256| sha256.to_string()),
                ..Default::default()
            }
        }).map(|image| {
            if let Some(captures) = parse_image_name.captures(&image.fqn) {
                Image {
                    name: String::from(&captures["image"]),
                    ..image
                }
            } else {
                image
            }
        }).ok_or(anyhow!("could not parse {} as an image", &image))
    }

    fn name_with_tag(&self) -> String {
        format!(
            "{}{}",
            self.name,
            self.tag.as_ref().map(|tag| format!(":{}", tag)).unwrap_or_default(),
        )
    }

    fn fully_qualified_name(&self) -> String {
        format!(
            "{}{}",
            self.name_with_tag(),
            self.sha256.as_ref().map(|sha256| format!("@sha256:{}", sha256)).unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_host() -> Result<()> {
        let image = Image::new("example.com/image:tag")?;
        assert_eq!(image.registry, Some("example.com".into()));

        let image = Image::new("example.com:5000/image:tag")?;
        assert_eq!(image.registry, Some("example.com:5000".into()));

        let image = Image::new("10.0.0.100/image:tag")?;
        assert_eq!(image.registry, Some("10.0.0.100".into()));

        let image = Image::new("10.0.0.100:5000/image:tag")?;
        assert_eq!(image.registry, Some("10.0.0.100:5000".into()));

        Ok(())
    }

    #[test]
    fn parse_image() -> Result<()> {
        let image = Image::new("image")?;
        assert_eq!(image.name, "image");

        let image = Image::new("image:tag")?;
        assert_eq!(image.name, "image");

        let image = Image::new("example.com/image")?;
        assert_eq!(image.name, "image");

        let image = Image::new("example.com/image:tag")?;
        assert_eq!(image.name, "image");

        let image = Image::new("example.com:5000/image")?;
        assert_eq!(image.name, "image");

        let image = Image::new("example.com:5000/image:tag")?;
        assert_eq!(image.name, "image");

        let image = Image::new("10.0.0.100/image")?;
        assert_eq!(image.name, "image");

        let image = Image::new("10.0.0.100/image:tag")?;
        assert_eq!(image.name, "image");

        let image = Image::new("10.0.0.100:5000/image")?;
        assert_eq!(image.name, "image");

        let image = Image::new("10.0.0.100:5000/image:tag")?;
        assert_eq!(image.name, "image");

        Ok(())
    }

    #[test]
    fn parse_fully_qualified_image() -> Result<()> {
        let image = Image::new("example.com/image:tag@sha256:3fc9b689459d738f8c88a3a48aa9e33542016b7a4052e001aaa536fca74813cb")?;
        assert_eq!(image.registry, Some("example.com".into()));
        assert_eq!(image.name, "image");
        assert_eq!(image.tag, Some("tag".into()));
        assert_eq!(image.sha256, Some("3fc9b689459d738f8c88a3a48aa9e33542016b7a4052e001aaa536fca74813cb".into()));

        Ok(())
    }
}
