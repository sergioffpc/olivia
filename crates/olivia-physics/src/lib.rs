//! The physics simulation, a thin wrapper over `rapier2d` in metres.

mod event;
mod world;

pub use rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Ball, Capsule, CoefficientCombineRule, Collider,
    ColliderBuilder, ColliderHandle, ConvexPolygon, Cuboid, Group, HalfSpace, HeightField,
    InteractionGroups, InteractionTestMode, LockedAxes, MassProperties, Polyline, Pose, RigidBody,
    RigidBodyBuilder, RigidBodyHandle, RigidBodyType, Rotation, Segment, Shape, SharedShape,
    TriMesh, Triangle, TypedShape,
};

pub use event::Event;
pub use world::World;
