use async_zip::base::read::mem::ZipFileReader;
use derive_getters::Getters;
use futures_util::io;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use tokio::fs;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use reqwest::Client;

pub enum Platform {
    Windows,
    Mac,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => f.write_str("windows"),
            Self::Mac => f.write_str("osx"),
        }
    }
}

#[derive(Serialize, Deserialize, Getters, Debug)]
#[serde(rename_all = "camelCase")]
struct ComponentManifest {
    application_type: String,
    #[serde(rename = "baseURI")]
    base_uri: String,
    files: Vec<ComponentFile>,
    force_update: bool,
    project_name: String,
    symlinks: Vec<ComponentSymlink>,
    timestamp: String,
    version: String,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
struct ComponentFile {
    hash: String,
    path: String,
    resource: String,
    sha256: String,
    size: u32,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
struct ComponentSymlink {
    path: String,
    target: String,
}

pub async fn get_peer(
    reqwest_client: &Client,
    dest_path: PathBuf,
    platform: Platform,
) -> Result<(), reqwest::Error> {
    let version_path = dest_path.join(format!(".peer-version-{}", platform.to_string()));
    log::debug!("Checking for peer updates");
    let url = format!(
        "https://cfg.gog.com/desktop-galaxy-peer/7/master/files-{}.json",
        platform.to_string()
    );

    let manifest_res = reqwest_client.get(url).send().await?;
    let manifest: ComponentManifest = manifest_res.json().await?;

    if dest_path.exists() {
        if let Ok(version_str) = fs::read_to_string(&version_path).await {
            if version_str == manifest.version && !manifest.force_update {
                return Ok(());
            }
        }
    } else {
        fs::create_dir_all(&dest_path)
            .await
            .expect("Failed to create directory");
    }

    // Download
    let n_of_files = manifest.files().len();
    for (i, file) in manifest.files().iter().enumerate() {
        log::info!("Downloading peer file {} of {}", i + 1, n_of_files);
        let url = format!("{}/{}", manifest.base_uri(), file.resource());
        let response = reqwest_client.get(url).send().await?;
        let data = response.bytes().await?;

        let zip = ZipFileReader::new(data.to_vec()).await.unwrap();

        let file_path = dest_path.join(file.path());
        let parent = file_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent)
                .await
                .expect("Failed to create directory");
        }

        let mut reader = zip.reader_without_entry(0).await.unwrap();
        let file_handle = fs::File::create(file_path)
            .await
            .expect("Failed to write file");
        io::copy(&mut reader, &mut file_handle.compat_write())
            .await
            .expect("Failed to write file");
    }

    fs::write(version_path, manifest.version())
        .await
        .expect("Failed to write version");

    Ok(())
}
