use std::io::Cursor;
use std::sync::Arc;

pub(crate) use kira::backend::cpal::CpalBackend as Backend;
use kira::sound::FromFileError;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};

use crate::Error;

/// The handle of a playing [`Stream`].
pub(crate) type StreamHandle = StreamingSoundHandle<FromFileError>;

/// A sound for [`Audio::stream`](crate::Audio::stream): the compressed bytes, which each play
/// decodes through a cursor of its own.
#[derive(Clone, Debug)]
pub(crate) struct Stream(Arc<[u8]>);

impl Stream {
    pub(crate) fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        let stream = Self(Arc::from(bytes));
        stream.data().map_err(Error::undecodable_audio)?;
        Ok(stream)
    }

    /// The data for one play.
    pub(crate) fn data(&self) -> Result<StreamingSoundData<FromFileError>, FromFileError> {
        StreamingSoundData::from_cursor(Cursor::new(Arc::clone(&self.0)))
    }
}

/// The desktop's audio is never held back, so there is nothing to resume.
pub(crate) fn resume(_backend: &mut Backend) {}

/// The desktop keeps its default panic hook, so nothing calls this.
pub(crate) fn suspend_all() {}
