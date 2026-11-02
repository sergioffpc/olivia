use std::collections::HashSet;

use olivia_math::Vec2;
use serde::Deserialize;

use crate::Error;

const VERSION: u64 = 1;

/// Read on its own, so that a newer file with another shape still reports its version.
#[derive(Deserialize)]
pub(crate) struct Header {
    pub(crate) version: u64,
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

/// A valid skeleton.
#[derive(Debug)]
pub(crate) struct Skeleton {
    /// The joints in the order of the file.
    pub(crate) joints: Vec<Joint>,
    /// The index of each joint, with every joint after the one it hangs from.
    pub(crate) walk: Vec<usize>,
}

/// A joint of a valid skeleton.
#[derive(Debug)]
pub(crate) struct Joint {
    /// Its name, which animations use.
    pub(crate) name: String,
    /// The image name of its part.
    pub(crate) part: String,
    /// Its pixel on its part's image.
    pub(crate) point: Vec2,
    /// The index of the joint it hangs from.
    pub(crate) parent: Option<usize>,
    /// Where it is from the joint it hangs from as drawn, in pixels.
    pub(crate) offset: Vec2,
}

/// Reads a `skeleton.json`, keeping the order of its joints.
pub(crate) fn parse(bytes: &[u8]) -> Result<Skeleton, Error> {
    let Header { version } = serde_json::from_slice(bytes)?;
    if version != VERSION {
        return Err(Error::UnknownVersion(version));
    }
    let File { joints: entries } = serde_json::from_slice(bytes)?;
    let walk = walk(&entries)?;
    let joint = |entry: &Entry| {
        let parent = |part: &String| entries.iter().position(|other| other.part == *part);
        let parent = entry.parent.as_ref().and_then(parent);
        let from_parent = |(parent, point): (usize, [f32; 2])| {
            Vec2::from(point) - Vec2::from(entries[parent].point)
        };
        Joint {
            name: entry.name.clone(),
            part: entry.part.clone(),
            point: entry.point.into(),
            parent,
            offset: parent
                .zip(entry.parent_point)
                .map_or(Vec2::ZERO, from_parent),
        }
    };
    Ok(Skeleton {
        joints: entries.iter().map(joint).collect(),
        walk,
    })
}

/// Walks the tree from the root, finding each joint from the one it hangs from.
fn walk(joints: &[Entry]) -> Result<Vec<usize>, Error> {
    let mut walk = vec![root(joints)?];
    let mut next = 0;
    while let Some(&parent) = walk.get(next) {
        next += 1;
        for (index, joint) in joints.iter().enumerate() {
            if joint.parent.as_ref() == Some(&joints[parent].part) {
                if joint.parent_point.is_none() {
                    let reason = format!("joint {:?} has no parent_point", joint.name);
                    return Err(invalid(reason));
                }
                walk.push(index);
            }
        }
    }
    if walk.len() < joints.len() {
        return Err(invalid("the joints form a cycle".to_owned()));
    }
    Ok(walk)
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
