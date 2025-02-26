use std::fs;

use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};

use crate::gui::{GdSfx, VersionType};
use crate::library::{parse_library, LibraryEntry, Library};
use crate::util::SFX_LIBRARY_FILE;

pub const GET_CUSTOM_CONTENT_URL: &str = "https://www.boomlings.com/database/getCustomContentURL.php";
pub const CDN_URL: &str = "https://geometrydashfiles.b-cdn.net";
pub const ENDPOINT_SFX_VERSION: &str = "sfx/sfxlibrary_version.txt";
pub const ENDPOINT_SFX_LIBRARY: &str = "sfx/sfxlibrary.dat";

impl GdSfx {
    pub fn get_cdn_url(&mut self, force: bool) -> Option<&String> {
        if !force && self.cdn_url.is_some() {
            return self.cdn_url.as_ref();
        }

        let request = Client::default()
            .post(GET_CUSTOM_CONTENT_URL)
            .header(USER_AGENT, "")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .ok()?;

        if request.status().is_success() {
            if let Ok(cdn_url) = request.text() {
                self.cdn_url = Some(cdn_url);
                return self.cdn_url.as_ref()
            }
        }
        None
    }

    #[allow(unused)]
    pub fn get_sfx_version(&mut self, force: bool) -> Option<VersionType> {
        if !force && self.sfx_version.is_some() {
            return self.sfx_version;
        }

        let cdn_url = self.get_cdn_url(force)?;

        self.sfx_version = Client::default()
            .get(format!("{cdn_url}/{ENDPOINT_SFX_VERSION}"))
            .send().ok()?
            .text().ok()?
            .parse().ok();

        self.sfx_version
    }

    pub fn get_sfx_library(&mut self, force: bool) -> Option<&Library> {
        if !force && SFX_LIBRARY_FILE.exists() {
            let sfx_data = fs::read(SFX_LIBRARY_FILE.as_path()).unwrap();
            let root = parse_library(&sfx_data);

            if self.sfx_version
                .map(|ver| ver.to_string() == root.sound_effects.name())
                .unwrap_or(false)
            {
                self.sfx_library = Some(root);
                return self.sfx_library.as_ref();
            }
        }

        let root = download_and_parse_library(self.get_cdn_url(force)?);
        self.sfx_library = Some(root);
        self.sfx_library.as_ref()
    }
}

fn download_and_parse_library(cdn_url: &str) -> Library {
    let sfx_data = Client::default()
        .get(format!("{cdn_url}/{ENDPOINT_SFX_LIBRARY}"))
        .send().unwrap()
        .bytes().unwrap();

    fs::write(SFX_LIBRARY_FILE.as_path(), &sfx_data).unwrap();
    parse_library(&sfx_data)
}

pub fn download_sfx(cdn_url: &str, sound: &LibraryEntry) -> Option<Vec<u8>> {
    let url = format!("{cdn_url}/sfx/{}", sound.filename());
    let response = Client::default()
        .get(url)
        .send().ok()?;

    if response.status().is_success() {
        Some(response.bytes().ok()?.to_vec())
    } else {
        None
    }
}
