use std::fmt;

use olivia_math::Vec2;
use olivia_physics::{ColliderBuilder, RigidBodyBuilder, RigidBodyHandle, World};

use crate::ecs::{self, Body, Entity, Transform};

/// The physics world, kept in step with the entities that have a body.
pub(crate) struct Simulation {
    pub(crate) physics: World,
    /// Every body Olivia made, and the entity it belongs to.
    bodies: Vec<(Entity, RigidBodyHandle)>,
}

impl Simulation {
    pub(crate) fn new(gravity: Vec2) -> Self {
        Self {
            physics: World::new(gravity),
            bodies: Vec::new(),
        }
    }

    /// Gives a body to the entities waiting for one, and removes the bodies of the ones gone.
    pub(crate) fn sync(&mut self, world: &mut ecs::World) {
        let physics = &mut self.physics;
        self.bodies.retain(|&(entity, handle)| {
            let exists = world.satisfies::<&Body>(entity);
            if !exists {
                physics.remove_body(handle);
            }
            exists
        });

        let mut waiting: Vec<Entity> = world
            .query::<Entity>()
            .with::<&RigidBodyBuilder>()
            .iter()
            .collect();
        waiting.extend(
            world
                .query::<Entity>()
                .with::<&ColliderBuilder>()
                .without::<&RigidBodyBuilder>()
                .iter(),
        );
        for entity in waiting {
            self.insert_body(world, entity);
        }
    }

    fn insert_body(&mut self, world: &mut ecs::World, entity: Entity) {
        let mut body = world
            .remove_one::<RigidBodyBuilder>(entity)
            .unwrap_or_else(|_| RigidBodyBuilder::fixed());
        if let Ok(transform) = world.get::<&Transform>(entity) {
            body = body
                .translation(transform.position)
                .rotation(transform.rotation);
        }
        let handle = self.physics.insert_body(body);
        if let Ok(collider) = world.remove_one::<ColliderBuilder>(entity) {
            let collider = collider.user_data(entity.to_bits().get().into());
            self.physics.insert_collider(collider, Some(handle));
        }
        world
            .insert_one(entity, Body(handle))
            .expect("the entity was just queried");
        self.bodies.push((entity, handle));
    }

    /// Advances the physics one fixed step and moves the entities with it.
    pub(crate) fn step(&mut self, world: &mut ecs::World) {
        self.sync(world);
        self.physics.step();
        for (Body(body), transform) in world.query_mut::<(&Body, &mut Transform)>() {
            let body = self.physics.body(*body);
            transform.position = body.translation();
            transform.rotation = body.rotation().angle();
        }
    }
}

impl fmt::Debug for Simulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Simulation")
            .field("physics", &self.physics)
            .finish_non_exhaustive()
    }
}
