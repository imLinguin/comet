use super::achievements::Achievement;

#[derive(Clone, Debug)]
pub enum OverlayPeerMessage {
    Achievement(Achievement),
    OpenWebPage(String),
    InvitationDialog(String),
    VisibilityChange(bool),
}
