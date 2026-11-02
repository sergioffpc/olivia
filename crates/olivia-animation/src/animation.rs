use std::sync::Arc;

use serde::Deserialize;

use crate::{Error, read};

/// Timed poses for joints, shared and stateless. Cheap to clone.
///
/// The value is its identity: clones are the same animation, and two parses of the same bytes are
/// not.
#[derive(Clone, Debug)]
pub struct Animation {
    file: Arc<File>,
}

/// An animation as the file has it, once checked.
#[derive(Debug, Deserialize)]
struct File {
    fps: u32,
    joints: Vec<String>,
    rows: Vec<Vec<f32>>,
}

impl Animation {
    /// Reads an animation file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Json`] if the file is malformed, [`Error::UnknownVersion`] if it has a
    /// version this reader does not know, or [`Error::InvalidAnimation`] if its `fps` is zero, it
    /// has no rows, or a row's length differs from `joints`.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let file: File = read(bytes)?;
        if file.fps == 0 {
            return Err(invalid("fps is zero".to_owned()));
        }
        if file.rows.is_empty() {
            return Err(invalid("there are no rows".to_owned()));
        }
        let wrong = |row: &Vec<f32>| row.len() != file.joints.len();
        if let Some(row) = file.rows.iter().position(wrong) {
            return Err(invalid(format!(
                "row {row} has a length other than the {} joints",
                file.joints.len()
            )));
        }
        Ok(Self {
            file: Arc::new(file),
        })
    }

    /// Whether `other` is this animation, rather than one that holds the same.
    pub(crate) fn same_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.file, &other.file)
    }

    /// The joints of the columns, in order.
    pub(crate) fn joints(&self) -> &[String] {
        &self.file.joints
    }

    /// The whole rows in `age` seconds, and how far it is into the next one.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the floor of a time that is not negative, and a fraction"
    )]
    fn row(&self, age: f64) -> (usize, f32) {
        let row = age.max(0.0) * f64::from(self.file.fps);
        (row as usize, row.fract() as f32)
    }

    /// Whether `age` seconds in has reached the last row.
    pub(crate) fn ended(&self, age: f64) -> bool {
        self.row(age).0 + 1 >= self.file.rows.len()
    }

    /// The two rows around `age` seconds in, and how far it is from the first to the second.
    pub(crate) fn rows(&self, age: f64, looping: bool) -> (&[f32], &[f32], f32) {
        let rows = &self.file.rows;
        let count = rows.len();
        let (row, between) = self.row(age);
        if looping {
            (&rows[row % count], &rows[(row + 1) % count], between)
        } else if row + 1 < count {
            (&rows[row], &rows[row + 1], between)
        } else {
            (&rows[count - 1], &rows[count - 1], 0.0)
        }
    }
}

/// How to play an animation: by default once, with a cut.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Play {
    pub(crate) looping: bool,
    pub(crate) crossfade: f32,
}

impl Play {
    /// Creates the default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether the animation loops, or else stops at its last row.
    #[must_use]
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets how many seconds the change to this animation takes. Zero is a cut.
    #[must_use]
    pub fn crossfade(mut self, seconds: f32) -> Self {
        self.crossfade = seconds;
        self
    }
}

fn invalid(reason: String) -> Error {
    Error::InvalidAnimation(reason)
}
