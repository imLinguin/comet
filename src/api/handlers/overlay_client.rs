use std::sync::Arc;

use crate::api::structs::UserInfo;
use crate::proto::galaxy_protocols_overlay_for_client::*;
use crate::proto::{common_utils::ProtoPayload, gog_protocols_pb};
use log::warn;
use protobuf::{Enum, Message};
use serde_json::json;

use super::{context::HandlerContext, MessageHandlingError, MessageHandlingErrorKind};

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &reqwest::Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::OVERLAY_FRONTEND_INIT_DATA_REQUEST.value() {
        overlay_data_request(payload, context, user_info, reqwest_client).await
    } else {
        warn!(
            "Received unsupported ov_client message type {}",
            message_type
        );
        Err(MessageHandlingError::new(
            MessageHandlingErrorKind::NotImplemented,
        ))
    }
}

async fn overlay_data_request(
    _payload: &ProtoPayload,
    _context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &reqwest::Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let game_id: String = std::env::var("HEROIC_APP_NAME").unwrap_or_default();
    let default_data = json! ({
        "id": "",
        "title": "Comet",
        "images": {
            "icon": "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/public/icon.png",
            "logo": "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/public/icon.png",
            "logo2x": "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/public/icon.png",
        }
    });
    let game_details = if !game_id.is_empty() {
        if let Ok(res) = reqwest_client
            .get(format!("https://api.gog.com/products/{}", game_id))
            .send()
            .await
        {
            if let Ok(mut res) = res.json::<serde_json::Value>().await {
                if let Some(serde_json::Value::Object(ref mut images)) = res.get_mut("images") {
                    for (_key, url_value) in images.iter_mut() {
                        if let serde_json::Value::String(url) = url_value {
                            if url.starts_with("//") {
                                *url_value = serde_json::Value::String(format!("https:{}", url));
                            }
                        }
                    }
                }
                res
            } else {
                default_data
            }
        } else {
            default_data
        }
    } else {
        default_data
    };

    let init_data = json!(
    {
      "Languages": [
        { "Code": "en", "EnglishName": "English", "NativeName": "English" },
        { "Code": "de", "EnglishName": "German", "NativeName": "Deutsch" },
        { "Code": "fr", "EnglishName": "French", "NativeName": "Français" },
        { "Code": "ru", "EnglishName": "Russian", "NativeName": "Русский" },
        { "Code": "pl", "EnglishName": "Polish", "NativeName": "Polski" },
        { "Code": "es", "EnglishName": "Spanish", "NativeName": "Español" },
        { "Code": "it", "EnglishName": "Italian", "NativeName": "Italiano" },
        { "Code": "jp", "EnglishName": "Japanese", "NativeName": "日本語" },
        { "Code": "ko", "EnglishName": "Korean", "NativeName": "한국어" },
        { "Code": "pt", "EnglishName": "Portuguese", "NativeName": "Português" },
        { "Code": "tr", "EnglishName": "Turkish", "NativeName": "Türkçe" },
        { "Code": "cz", "EnglishName": "Czech", "NativeName": "Čeština" },
        { "Code": "cn", "EnglishName": "Chinese", "NativeName": "中文" },
        { "Code": "hu", "EnglishName": "Hungarian", "NativeName": "Magyar" },
        { "Code": "nl", "EnglishName": "Dutch", "NativeName": "Nederlands" },
        { "Code": "ho", "EnglishName": "Hiri Motu", "NativeName": "Hiri Motu" },
        { "Code": "ro", "EnglishName": "Romanian", "NativeName": "Română" }
      ],
      "SettingsData": {
        "languageCode": crate::LOCALE.clone(),
        "notifChatMessage": true,
        "notifDownloadStatus": true,
        "notifDownloadStatus_overlay": true,
        "notifFriendInvite": true,
        "notifFriendOnline": true,
        "notifFriendStartsGame": true,
        "notifGameInvite": true,
        "notifSoundChatMessage": true,
        "notifSoundDownloadStatus": false,
        "notifSoundFriendInvite": true,
        "notifSoundFriendOnline": true,
        "notifSoundFriendStartsGame": true,
        "notifSoundGameInvite": true,
        "notifSoundVolume": 50,
        "showFriendsSidebar": true,
        "overlayNotificationsPosition": "bottom_right",
        "store": {}
      },
      "Config": {
        "Endpoints": {
          "api": "https://api.gog.com",
          "chat": "https://chat.gog.com",
          "externalAccounts": "https://external-accounts.gog.com",
          "externalUsers": "https://external-users.gog.com",
          "gameplay": "https://gameplay.gog.com",
          "gog": "https://embed.gog.com",
          "gogGalaxyStoreApi": "https://embed.gog.com",
          "notifications": "https://notifications.gog.com",
          "pusher": "https://notifications-pusher.gog.com",
          "library": "https://galaxy-library.gog.com",
          "presence": "https://presence.gog.com",
          "users": "https://users.gog.com",
          "redeem": "https://redeem.gog.com",
          "marketingSections": "https://marketing-sections.gog.com",
          "galaxyPromos": "https://galaxy-promos.gog.com",
          "remoteConfigurationHost": "https://remote-config.gog.com",
          "recommendations": "https://recommendations-api.gog.com",
          "overlayWeb": "https://overlay.gog.com",
          "OverlayWeb": "https://overlay.gog.com"
        },
        "GalaxyClientId": "46899977096215655",
        "ChangelogBasePath": "",
        "LoggingLevel": 5,
        "ClientVersions": { "Major": 2, "Minor": 0, "Build": 75, "Compilation": 1 }
      },
      "User": {
          "UserId": user_info.galaxy_user_id.clone()
      },
      "Game": {
          "ProductId": game_id,
          "ProductDetails": game_details
      }
    });

    let mut res = OverlayFrontendInitDataResponse::new();
    res.set_data(init_data.to_string());
    let payload = res
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let mut header = gog_protocols_pb::Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
    header.set_type(
        MessageType::OVERLAY_FRONTEND_INIT_DATA_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}
