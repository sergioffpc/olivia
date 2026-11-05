use olivia::{
    Result,
    animation::{Character, Play},
    ecs::{CSprite, CTransform, Entity, World},
    math::Vec2,
};

use super::{
    CActor,
    layout::{GOAL, GROUND, HELP_DISTANCE, WIDTH},
    zone,
};
use crate::{
    assets::{Assets, ROOT_HEIGHT},
    camera::VIEW,
    depth,
    sprite::sized,
    status::{CStatus, Outcome},
};

/// How long her calling takes to give way to her freed animation.
const CROSSFADE: f32 = 0.1;
/// The size of the cage the princess is held in.
const GOAL_SIZE: Vec2 = Vec2::new(2.2, 3.0);

/// The cage that saves the princess when the hero reaches it.
pub(super) struct CGoal;
/// The stretch close enough to the cage for the hero to hear the princess.
pub(super) struct CEarshot;
/// The princess, held in the cage.
struct CPrincess;

#[derive(Debug)]
pub(super) struct Goal {
    princess: Entity,
}

/// The cage standing on the ground at [`GOAL`], the princess in it calling for help, and the
/// stretch from which the hero hears her.
pub(super) fn spawn(world: &mut World, assets: &Assets) -> Result<Goal> {
    let Assets {
        animations,
        characters,
        textures,
        ..
    } = assets;
    let centre = GROUND - GOAL_SIZE.y / 2.0;
    let draw = sized(&textures.goal, GOAL_SIZE).depth(depth::GOAL);
    let sprite = CSprite::new(textures.goal.clone()).draw(draw);
    world.spawn((
        CActor,
        CGoal,
        CTransform::at(GOAL, centre),
        zone(GOAL_SIZE),
        sprite,
    ));

    let earshot = Vec2::new(WIDTH - (GOAL - HELP_DISTANCE), VIEW.y);
    world.spawn((
        CActor,
        CEarshot,
        CTransform::at(WIDTH - earshot.x / 2.0, VIEW.y / 2.0),
        zone(earshot),
    ));

    let mut princess = characters.princess.clone();
    // Fails if the animation names a joint the skeleton lacks, as with the default skeleton.
    princess.play(&animations.princess.calling, Play::new().looping(true))?;
    let princess = world.spawn((
        CActor,
        CPrincess,
        CTransform::at(GOAL, GROUND - ROOT_HEIGHT),
        princess,
    ));

    Ok(Goal { princess })
}

/// Frees the princess once the game is won: she plays her freed animation, once.
pub(super) fn update(
    world: &mut World,
    goal: &Goal,
    assets: &Assets,
    status: CStatus,
) -> Result<()> {
    if status.outcome == Outcome::Won {
        let freed = Play::new().crossfade(CROSSFADE);
        let mut princess = world.get::<&mut Character>(goal.princess)?;
        princess.play(&assets.animations.princess.freed, freed)?;
    }
    Ok(())
}
