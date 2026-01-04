use super::achievements::Achievement;

#[derive(Clone, Debug)]
pub enum OverlayPeerMessage {
    InitConnection(String),
    Achievement(Achievement),
    DisablePopups(Vec<u8>),
    OpenWebPage(String),
    InvitationDialog(String),
    VisibilityChange(bool),
    GameJoin((u64, String, String)),
}
