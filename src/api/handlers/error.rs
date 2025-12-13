#[derive(Debug)]
pub enum MessageHandlingErrorKind {
    NotImplemented,
    Ignored,
    Unauthorized,
    IO(tokio::io::Error),
    DB(sqlx::Error),
    Network(reqwest::Error),
    Proto(protobuf::Error),
    Json(serde_json::Error),
}

#[derive(Debug)]
pub struct MessageHandlingError {
    pub kind: MessageHandlingErrorKind,
}

impl std::error::Error for MessageHandlingError {}

impl MessageHandlingError {
    pub fn new(kind: MessageHandlingErrorKind) -> MessageHandlingError {
        MessageHandlingError { kind }
    }

    pub fn not_implemented() -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::NotImplemented,
        }
    }

    pub fn ignored() -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::Ignored,
        }
    }

    pub fn unauthorized() -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::Unauthorized,
        }
    }

    pub fn io(err: tokio::io::Error) -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::IO(err),
        }
    }

    pub fn db(err: sqlx::Error) -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::DB(err),
        }
    }

    pub fn network(err: reqwest::Error) -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::Network(err),
        }
    }

    pub fn proto(err: protobuf::Error) -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::Proto(err),
        }
    }

    pub fn json(err: serde_json::Error) -> MessageHandlingError {
        MessageHandlingError {
            kind: MessageHandlingErrorKind::Json(err),
        }
    }
}

impl std::fmt::Display for MessageHandlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
