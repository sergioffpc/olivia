use std::collections::HashMap;
use std::fmt;

use olivia_math::Vec2;
use rapier2d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderSet, DebugRenderMode, DebugRenderPipeline, DebugRenderStyle,
    ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase,
    PhysicsPipeline, QueryFilter, QueryPipeline, Ray, RigidBodySet, SoftBodySet,
};

use crate::debug::Lines;
use crate::event::Collector;
use crate::{Collider, ColliderHandle, Event, Pose, RigidBody, RigidBodyHandle, Shape};

/// The seconds one [`World::step`] simulates, at 60 Hz.
const FIXED_STEP: f32 = 1.0 / 60.0;

/// The physics simulation a game owns, in metres.
pub struct World {
    gravity: Vec2,
    parameters: IntegrationParameters,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    soft_bodies: SoftBodySet,
    ccd_solver: CCDSolver,
    events: Vec<Event>,
    /// Each body's pose at the start of the last step.
    previous: HashMap<RigidBodyHandle, Pose>,
}

impl World {
    /// Creates an empty world with `gravity` in metres per second squared.
    #[must_use]
    pub fn new(gravity: Vec2) -> Self {
        Self {
            gravity,
            parameters: IntegrationParameters {
                dt: FIXED_STEP,
                ..IntegrationParameters::default()
            },
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            soft_bodies: SoftBodySet::new(),
            ccd_solver: CCDSolver::new(),
            events: Vec::new(),
            previous: HashMap::new(),
        }
    }

    /// Advances the simulation by one fixed step. Call it once per update.
    pub fn step(&mut self) {
        self.previous.clear();
        self.previous.extend(
            self.bodies
                .iter()
                .map(|(handle, body)| (handle, *body.position())),
        );
        let collector = Collector::default();
        self.pipeline.step(
            self.gravity,
            &self.parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.soft_bodies,
            &mut self.ccd_solver,
            &(),
            &collector,
        );
        self.events = collector.into_events();
    }

    /// What happened during the last step, valid until the next one.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Adds a body to the world.
    pub fn insert_body(&mut self, body: impl Into<RigidBody>) -> RigidBodyHandle {
        self.bodies.insert(body)
    }

    /// Removes a body and the colliders attached to it.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        self.bodies
            .remove(
                handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                &mut self.soft_bodies,
                true,
            )
            .expect("stale body handle");
    }

    /// The body behind `handle`.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    #[must_use]
    pub fn body(&self, handle: RigidBodyHandle) -> &RigidBody {
        &self.bodies[handle]
    }

    /// The body behind `handle`, to change it.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    pub fn body_mut(&mut self, handle: RigidBodyHandle) -> &mut RigidBody {
        &mut self.bodies[handle]
    }

    /// Adds a collider to the world, attached to `parent` if there is one.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is stale.
    pub fn insert_collider(
        &mut self,
        collider: impl Into<Collider>,
        parent: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        match parent {
            Some(parent) => self
                .colliders
                .insert_with_parent(collider, parent, &mut self.bodies),
            None => self.colliders.insert(collider),
        }
    }

    /// Removes a collider.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.colliders
            .remove(
                handle,
                &mut self.islands,
                &mut self.bodies,
                &mut self.soft_bodies,
                true,
            )
            .expect("stale collider handle");
    }

    /// The collider behind `handle`.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    #[must_use]
    pub fn collider(&self, handle: ColliderHandle) -> &Collider {
        &self.colliders[handle]
    }

    /// The collider behind `handle`, to change it.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale.
    pub fn collider_mut(&mut self, handle: ColliderHandle) -> &mut Collider {
        &mut self.colliders[handle]
    }

    /// Where a body is between the last two steps, for `factor` from 0 to 1. Pass the frame's
    /// interpolation factor. A body inserted since the last step stays where it is.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale or `factor` is outside 0 to 1.
    #[must_use]
    pub fn interpolated_position(&self, handle: RigidBodyHandle, factor: f32) -> Vec2 {
        self.interpolated(handle, factor).translation
    }

    /// The angle in radians of a body between the last two steps, turning the short way, for
    /// `factor` from 0 to 1. Pass the frame's interpolation factor. A body inserted since the last
    /// step stays where it is.
    ///
    /// # Panics
    ///
    /// Panics if `handle` is stale or `factor` is outside 0 to 1.
    #[must_use]
    pub fn interpolated_angle(&self, handle: RigidBodyHandle, factor: f32) -> f32 {
        self.interpolated(handle, factor).rotation.angle()
    }

    fn interpolated(&self, handle: RigidBodyHandle, factor: f32) -> Pose {
        assert!(
            (0.0..=1.0).contains(&factor),
            "factor {factor} outside 0 to 1"
        );
        let current = self
            .bodies
            .get(handle)
            .expect("stale body handle")
            .position();
        self.previous
            .get(&handle)
            .unwrap_or(current)
            .lerp(current, factor)
    }

    /// The nearest collider along a ray from `origin` and how many metres away it is, within
    /// `max_distance`. `direction` needs no particular length. A ray starting inside a collider
    /// hits it at 0.
    ///
    /// A collider inserted or moved since the last step can be missed.
    ///
    /// # Panics
    ///
    /// Panics if `direction` is zero.
    #[must_use]
    pub fn ray_cast(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_distance: f32,
    ) -> Option<(ColliderHandle, f32)> {
        let direction = direction.try_normalize().expect("zero ray direction");
        self.queries()
            .cast_ray(&Ray::new(origin, direction), max_distance, true)
    }

    /// The colliders containing `point`, in no particular order.
    ///
    /// A collider inserted or moved since the last step can be missed.
    #[must_use]
    pub fn overlap_point(&self, point: Vec2) -> Vec<ColliderHandle> {
        self.queries()
            .intersect_point(point)
            .map(|(handle, _)| handle)
            .collect()
    }

    /// The colliders touching `shape` placed at `pose`, in no particular order.
    ///
    /// A collider inserted or moved since the last step can be missed.
    #[must_use]
    pub fn overlap_shape(&self, shape: &dyn Shape, pose: Pose) -> Vec<ColliderHandle> {
        self.queries()
            .intersect_shape(pose, shape)
            .map(|(handle, _)| handle)
            .collect()
    }

    /// The segments outlining every collider and its bounding box, in metres, to draw however the
    /// game likes. Bodies show the poses of the last step, not interpolated ones.
    ///
    /// Each call builds its render from scratch, so only a game that draws them pays for them.
    #[must_use]
    pub fn debug_lines(&self) -> Vec<[Vec2; 2]> {
        let mut lines = Lines::default();
        DebugRenderPipeline::new(
            DebugRenderStyle::default(),
            DebugRenderMode::COLLIDER_SHAPES | DebugRenderMode::COLLIDER_AABBS,
        )
        .render_colliders(&mut lines, &self.bodies, &self.colliders);
        lines.0
    }

    fn queries(&self) -> QueryPipeline<'_> {
        self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default(),
        )
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("gravity", &self.gravity)
            .field("bodies", &self.bodies.len())
            .field("colliders", &self.colliders.len())
            .finish_non_exhaustive()
    }
}
