use super::achievements::Achievement;

#[derive(Clone, Debug)]
pub enum OverlayPeerMessage {
    Achievement(Achievement),
    DisablePopups(Vec<u8>),
    OpenWebPage(String),
    InvitationDialog(String),
    VisibilityChange(bool),
    GameJoin((u64, String, String)),
}
