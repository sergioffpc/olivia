use std::cell::RefCell;
use std::convert::Infallible;
use std::io::Cursor;
use std::rc::{Rc, Weak};

use kira::backend::Renderer;
use kira::backend::cpal::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use kira::backend::cpal::cpal::{self, Device, StreamConfig};
use kira::backend::cpal::{CpalBackendSettings, Error as CpalError};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use send_wrapper::SendWrapper;
use tracing::warn;

use crate::Error;

thread_local! {
    /// Every output started, which a panic hook reaches though it cannot reach `Audio`.
    static STREAMS: RefCell<Vec<Weak<cpal::Stream>>> = const { RefCell::new(Vec::new()) };
}

/// The handle of a playing [`Stream`].
pub(crate) type StreamHandle = StaticSoundHandle;

/// A sound for [`Audio::stream`](crate::Audio::stream). The web has no thread to decode on while
/// playing, so it is decoded up front.
#[derive(Clone, Debug)]
pub(crate) struct Stream(Box<StaticSoundData>);

impl Stream {
    pub(crate) fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        let data =
            StaticSoundData::from_cursor(Cursor::new(bytes)).map_err(Error::undecodable_audio)?;
        Ok(Self(Box::new(data)))
    }

    /// The data for one play.
    #[expect(clippy::unnecessary_wraps, reason = "matches the desktop version")]
    pub(crate) fn data(&self) -> Result<StaticSoundData, Infallible> {
        Ok((*self.0).clone())
    }
}

/// A copy of `kira`'s `cpal` backend for the web, which also keeps the stream where [`resume`]
/// and [`suspend_all`] reach it.
pub(crate) struct Backend {
    state: SendWrapper<State>,
}

enum State {
    Empty,
    Uninitialized {
        device: Device,
        config: StreamConfig,
    },
    Initialized {
        stream: Rc<cpal::Stream>,
    },
}

impl kira::backend::Backend for Backend {
    type Settings = CpalBackendSettings;
    type Error = CpalError;

    fn setup(
        settings: CpalBackendSettings,
        _internal_buffer_size: usize,
    ) -> Result<(Self, u32), CpalError> {
        let device = match settings.device {
            Some(device) => device,
            None => cpal::default_host()
                .default_output_device()
                .ok_or(CpalError::NoDefaultOutputDevice)?,
        };
        let config = match settings.config {
            Some(config) => config,
            None => device.default_output_config()?.config(),
        };
        let sample_rate = config.sample_rate;
        let state = SendWrapper::new(State::Uninitialized { device, config });
        Ok((Self { state }, sample_rate))
    }

    fn start(&mut self, mut renderer: Renderer) -> Result<(), CpalError> {
        let State::Uninitialized { device, config } =
            std::mem::replace(&mut *self.state, State::Empty)
        else {
            panic!("the backend starts only once");
        };
        let channels = config.channels;
        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _| {
                renderer.on_start_processing();
                renderer.process(data, channels);
            },
            |_| {},
            None,
        )?;
        // Before a user gesture, the browser keeps it suspended until `resume`.
        stream.play()?;
        let stream = Rc::new(stream);
        STREAMS.with_borrow_mut(|streams| {
            streams.retain(|stream| stream.strong_count() > 0);
            streams.push(Rc::downgrade(&stream));
        });
        *self.state = State::Initialized { stream };
        Ok(())
    }
}

/// Resumes the `AudioContext`, which works only inside the handler of a user gesture.
pub(crate) fn resume(backend: &mut Backend) {
    if let State::Initialized { stream } = &*backend.state
        && let Err(error) = stream.play()
    {
        warn!(%error, "audio could not resume");
    }
}

/// Suspends every `AudioContext` started.
pub(crate) fn suspend_all() {
    STREAMS.with(|streams| {
        let Ok(streams) = streams.try_borrow() else {
            return;
        };
        for stream in streams.iter().filter_map(Weak::upgrade) {
            // A panic hook has nowhere left to report to.
            let _ = stream.pause();
        }
    });
}
