use std::io;

use crate::client::ClientMessage;

error_set::error_set! {
    LtrsError := {
        LtrsError(lt_rs::errors::LtrsError)
    }

    TokioSyncError := {
        SendError(tokio::sync::mpsc::error::SendError<ClientMessage>),
        RecvError(tokio::sync::oneshot::error::RecvError)
    }

    IoError := {
        IoError(io::Error)
    }

    LoadTorrentError := LtrsError || TokioSyncError || IoError

    TorrentError :=  LtrsError || TokioSyncError || {
        TorrentNotFound,
        TorrentHandleNotFound
    }

}
