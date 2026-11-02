use std::sync::{Mutex, PoisonError};

use rapier2d::prelude::{
    ColliderSet, CollisionEvent, ContactPair, EventHandler, Real, RigidBodySet, SoftBodySet,
    SoftBodyTearEvent,
};

use crate::ColliderHandle;

/// What happened between two colliders during a step.
///
/// A collider reports only with [`ActiveEvents::COLLISION_EVENTS`](crate::ActiveEvents) set, and
/// a pair reports if either collider does.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The colliders began touching.
    ContactStart(ColliderHandle, ColliderHandle),
    /// The colliders stopped touching. Removing a collider that was touching another stops the
    /// contact, so either handle can be stale.
    ContactStop(ColliderHandle, ColliderHandle),
}

/// Gathers the events of one step. `rapier2d` needs a `Sync` handler, hence the lock.
#[derive(Default)]
pub(crate) struct Collector(Mutex<Vec<Event>>);

impl Collector {
    pub(crate) fn into_events(self) -> Vec<Event> {
        self.0.into_inner().unwrap_or_else(PoisonError::into_inner)
    }
}

impl EventHandler for Collector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        let event = match event {
            CollisionEvent::Started(a, b, _) => Event::ContactStart(a, b),
            CollisionEvent::Stopped(a, b, _) => Event::ContactStop(a, b),
        };
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &ContactPair,
        _total_force_magnitude: Real,
    ) {
    }

    fn handle_soft_body_tear_event(&self, _soft_bodies: &SoftBodySet, _event: &SoftBodyTearEvent) {}
}
