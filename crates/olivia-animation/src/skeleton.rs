use std::collections::HashSet;

use olivia_math::Vec2;
use serde::Deserialize;

use crate::Error;

const VERSION: u64 = 1;

/// Read on its own, so that a newer file with another shape still reports its version.
#[derive(Deserialize)]
struct Header {
    version: u64,
}

#[derive(Deserialize)]
struct File {
    joints: Vec<Entry>,
}

/// A joint as the file has it.
#[derive(Deserialize)]
struct Entry {
    name: String,
    part: String,
    point: [f32; 2],
    parent: Option<String>,
    parent_point: Option<[f32; 2]>,
}

/// A joint of a valid skeleton.
#[derive(Debug)]
pub(crate) struct Joint {
    /// The image name of its part.
    pub(crate) part: String,
    /// Its pixel on its part's image.
    pub(crate) point: Vec2,
    /// Where it is from the root joint as drawn, in pixels.
    pub(crate) offset: Vec2,
}

/// Reads a `skeleton.json`, keeping the order of its joints.
pub(crate) fn parse(bytes: &[u8]) -> Result<Vec<Joint>, Error> {
    let Header { version } = serde_json::from_slice(bytes)?;
    if version != VERSION {
        return Err(Error::UnknownVersion(version));
    }
    let File { joints } = serde_json::from_slice(bytes)?;
    let offsets = offsets(&joints)?;
    let joint = |(entry, offset): (Entry, Vec2)| Joint {
        part: entry.part,
        point: entry.point.into(),
        offset,
    };
    Ok(joints.into_iter().zip(offsets).map(joint).collect())
}

/// Walks the tree from the root, finding each joint from the one it hangs from.
fn offsets(joints: &[Entry]) -> Result<Vec<Vec2>, Error> {
    let root = root(joints)?;
    let mut offsets = vec![None; joints.len()];
    offsets[root] = Some(Vec2::ZERO);
    let mut hung = vec![root];
    while let Some(parent) = hung.pop() {
        for (index, joint) in joints.iter().enumerate() {
            if joint.parent.as_ref() == Some(&joints[parent].part) {
                let parent_point = joint.parent_point.ok_or_else(|| {
                    invalid(format!("joint {:?} has no parent_point", joint.name))
                })?;
                let from_parent = Vec2::from(parent_point) - Vec2::from(joints[parent].point);
                offsets[index] = offsets[parent].map(|offset| offset + from_parent);
                hung.push(index);
            }
        }
    }
    let offsets: Option<Vec<Vec2>> = offsets.into_iter().collect();
    offsets.ok_or_else(|| invalid("the joints form a cycle".to_owned()))
}

/// Finds the only joint with no parent, once names, parts and parents are known to be sound.
fn root(joints: &[Entry]) -> Result<usize, Error> {
    let mut names = HashSet::new();
    let mut parts = HashSet::new();
    for joint in joints {
        if !names.insert(&joint.name) {
            return Err(invalid(format!("two joints are named {:?}", joint.name)));
        }
        if !parts.insert(&joint.part) {
            return Err(invalid(format!(
                "two joints have the part {:?}",
                joint.part
            )));
        }
    }
    for parent in joints.iter().filter_map(|joint| joint.parent.as_ref()) {
        if !parts.contains(parent) {
            return Err(invalid(format!("unknown parent {parent:?}")));
        }
    }

    let mut roots = (0..joints.len()).filter(|&index| joints[index].parent.is_none());
    match (roots.next(), roots.next()) {
        (Some(root), None) => Ok(root),
        (None, _) => Err(invalid("no joint is the root".to_owned())),
        (Some(_), Some(_)) => Err(invalid("more than one joint is the root".to_owned())),
    }
}

fn invalid(reason: String) -> Error {
    Error::InvalidSkeleton(reason)
}
