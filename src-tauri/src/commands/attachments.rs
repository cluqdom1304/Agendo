//! Commands for the files of the notes (D-47). The bytes travel as the raw body of the request
//! and the raw body of the response, not as JSON, so a large image is not inflated on the way.

use std::{path::PathBuf, sync::Arc};

use serde::Serialize;
use tauri::{
    AppHandle, State,
    ipc::{InvokeBody, Request, Response},
};

use crate::{
    attachments::{self, DIR_NAME},
    error::{AppError, AppResult},
    repo::attachments::Attachment,
    state::{AppState, run_blocking},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDto {
    pub id: String,
    pub mime: String,
    pub size_bytes: i64,
}

impl From<Attachment> for AttachmentDto {
    fn from(value: Attachment) -> Self {
        Self {
            id: value.id,
            mime: value.mime,
            size_bytes: value.size_bytes,
        }
    }
}

pub(crate) fn dir(app: &AppHandle) -> AppResult<PathBuf> {
    crate::data_dir(app)
        .map(|dir| dir.join(DIR_NAME))
        .map_err(|error| AppError::Io(format!("no se encontró la carpeta de datos: {error}")))
}

#[tauri::command]
pub async fn attachment_add(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    request: Request<'_>,
) -> Result<AttachmentDto, String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err(AppError::Validation("se esperaban los bytes del fichero".into()).to_string());
    };
    let bytes = bytes.clone();
    let dir = dir(&app)?;
    run_blocking(state.inner().clone(), move |state| {
        attachments::add(state, &dir, &bytes).map(AttachmentDto::from)
    })
    .await
}

#[tauri::command]
pub async fn attachment_read(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Response, String> {
    let dir = dir(&app)?;
    run_blocking(state.inner().clone(), move |state| {
        attachments::read(state, &dir, &id).map(|(_, bytes)| bytes)
    })
    .await
    .map(Response::new)
}
