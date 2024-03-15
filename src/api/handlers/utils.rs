use crate::api::gog::leaderboards::get_leaderboards_entries;
use crate::api::handlers::context::HandlerContext;
use crate::api::structs::IDType;
use log::{debug, warn};
use protobuf::{Enum, Message};
use reqwest::Client;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::proto::galaxy_protocols_communication_service::get_leaderboard_entries_response::LeaderboardEntry;
use crate::proto::galaxy_protocols_communication_service::{
    GetLeaderboardEntriesResponse, MessageType,
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

    let mut payload = match leaderboard_response {
        Ok(results) => {
            let mut data = GetLeaderboardEntriesResponse::new();
            data.set_leaderboard_entry_total_count(results.leaderboard_entry_total_count);
            data.leaderboard_entries
                .extend(results.items.iter().map(|item| {
                    let mut new_entry = LeaderboardEntry::new();
                    let user_id: u64 = item.user_id.parse().unwrap();
                    let user_id = (IDType::IdTypeUser as u64) << 56 | user_id;
                    new_entry.set_user_id(user_id);
                    new_entry.set_score(item.score);
                    new_entry.set_rank(item.rank);
                    new_entry
                }));
            data.write_to_bytes().unwrap()
        }
        Err(err) => {
            warn!("Leaderboards request error: {}", err);
            if err.is_status() {
                if err.status().unwrap() == reqwest::StatusCode::NOT_FOUND {
                    header
                        .mut_special_fields()
                        .mut_unknown_fields()
                        .add_varint(101, 404);
                }
            }
            Vec::new()
        }
    };

    header.set_size(payload.len().try_into().unwrap());

    ProtoPayload { header, payload }
}
