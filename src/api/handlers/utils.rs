use crate::api::gog;
use crate::api::gog::leaderboards::get_leaderboards_entries;
use crate::api::handlers::context::HandlerContext;
use crate::api::handlers::error::{MessageHandlingError, MessageHandlingErrorKind};
use crate::api::structs::IDType;
use crate::db;
use log::{debug, warn};
use protobuf::{Enum, Message};
use reqwest::Client;
use base64::prelude::*;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::proto::galaxy_protocols_communication_service::get_leaderboard_entries_response::LeaderboardEntry;
use crate::proto::galaxy_protocols_communication_service::{
    get_leaderboards_response, DisplayType, GetLeaderboardEntriesResponse, GetLeaderboardsResponse,
    MessageType, SortMethod,
};
use crate::proto::gog_protocols_pb::Header;
use crate::proto::{common_utils::ProtoPayload, gog_protocols_pb};

pub async fn parse_payload(
    h_size: u16,
    socket: &mut TcpStream,
) -> Result<ProtoPayload, tokio::io::Error> {
    let h_size: usize = h_size.into();
    let mut buffer: Vec<u8> = vec![0; h_size];

    socket.read_exact(&mut buffer).await?;

    let header = gog_protocols_pb::Header::parse_from_bytes(&buffer)?;

    let size: usize = header.size().try_into().unwrap();
    buffer.resize(size, 0);
    debug!(
        "Reading payload size {} to buffer of size {}",
        size,
        buffer.len()
    );
    socket.read_exact(&mut buffer).await?;
    Ok(ProtoPayload {
        header,
        payload: buffer,
    })
}

pub async fn handle_leaderboards_query<I, K, V>(
    context: &mut HandlerContext,
    reqwest_client: &Client,
    params: I,
) -> Result<ProtoPayload, MessageHandlingError>
where
    I: IntoIterator<Item = (K, V)> + Clone,
    K: AsRef<str>,
    V: AsRef<str> + std::fmt::Display,
{
    let leaderboards_network =
        gog::leaderboards::get_leaderboards(context, reqwest_client, params.clone()).await;

    let leaderboards = match leaderboards_network {
        Ok(ld) => ld,
        Err(_) => db::gameplay::get_leaderboards_defs(context, params)
            .await
            .unwrap_or_default(),
    };

    if let Err(err) = super::db::gameplay::update_leaderboards(&context, &leaderboards).await {
        log::error!("Failed to save leaderboards definitions {}", err);
    }

    let proto_defs = leaderboards.iter().map(|entry| {
        let mut new_def = get_leaderboards_response::LeaderboardDefinition::new();
        let display_type = match entry.display_type().as_str() {
            "numeric" => DisplayType::DISPLAY_TYPE_NUMERIC,
            "time_seconds" => DisplayType::DISPLAY_TYPE_TIME_SECONDS,
            "time_milliseconds" => DisplayType::DISPLAY_TYPE_TIME_MILLISECONDS,
            _ => DisplayType::DISPLAY_TYPE_UNDEFINED,
        };
        let sort_method = match entry.sort_method().as_str() {
            "asc" => SortMethod::SORT_METHOD_ASCENDING,
            "desc" => SortMethod::SORT_METHOD_DESCENDING,
            _ => SortMethod::SORT_METHOD_UNDEFINED,
        };

        new_def.set_key(entry.key().clone());
        new_def.set_name(entry.name().clone());
        new_def.set_leaderboard_id(entry.id().parse().unwrap());
        new_def.set_display_type(display_type);
        new_def.set_sort_method(sort_method);

        new_def
    });

    let mut payload_data = GetLeaderboardsResponse::new();
    payload_data.leaderboard_definitions.extend(proto_defs);

    let mut header = Header::new();
    header.set_type(
        MessageType::GET_LEADERBOARDS_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    let payload = payload_data
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}

pub async fn handle_leaderboard_entries_request<I, K, V>(
    context: &mut HandlerContext,
    reqwest_client: &Client,
    leaderboard_id: u64,
    params: I,
) -> ProtoPayload
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut header = Header::new();
    header.set_type(
        MessageType::GET_LEADERBOARD_ENTRIES_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    let leaderboard_response =
        get_leaderboards_entries(context, reqwest_client, leaderboard_id, params).await;

    let payload = match leaderboard_response {
        Ok(results) => {
            let mut data = GetLeaderboardEntriesResponse::new();
            data.set_leaderboard_entry_total_count(results.leaderboard_entry_total_count);
            data.leaderboard_entries
                .extend(results.items.iter().map(|item| {
                    let mut new_entry = LeaderboardEntry::new();
                    let user_id: u64 = item.user_id.parse().unwrap();
                    let user_id = IDType::User(user_id);
                    new_entry.set_user_id(user_id.value());
                    new_entry.set_score(item.score);
                    new_entry.set_rank(item.rank);
                    if let Some(details) = &item.details {
                        if let Ok(details) = BASE64_STANDARD_NO_PAD.decode(details) {
                            new_entry.set_details(details)
                        }
                    }
                    new_entry
                }));
            data.write_to_bytes().unwrap()
        }
        Err(err) => {
            warn!("Leaderboards request error: {}", err);
            if err.is_status() && err.status().unwrap() == reqwest::StatusCode::NOT_FOUND {
                header
                    .mut_special_fields()
                    .mut_unknown_fields()
                    .add_varint(101, 404);
            }
            Vec::new()
        }
    };

    header.set_size(payload.len().try_into().unwrap());

    ProtoPayload { header, payload }
}
