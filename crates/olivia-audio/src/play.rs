/// How to play a sound, built with chained setters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Play {
    pub(crate) volume: f32,
    pub(crate) pan: f32,
    pub(crate) pitch: f32,
    pub(crate) looping: bool,
}

impl Default for Play {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            looping: false,
        }
    }
}

impl Play {
    /// The default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the volume: linear from `0.0` to `1.0`, and above `1.0` amplifies. The default is `1.0`.
    #[must_use]
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Sets the pan, from `-1.0` (left) to `1.0` (right). The default is `0.0`.
    #[must_use]
    pub fn pan(mut self, pan: f32) -> Self {
        self.pan = pan;
        self
    }

    /// Sets the playback rate, which also changes the speed. The default is `1.0`.
    #[must_use]
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    /// Sets whether the whole track repeats until stopped. The default is `false`.
    #[must_use]
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}
