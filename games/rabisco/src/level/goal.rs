//! A jaula no fim do nível, onde a princesa está presa à espera do herói.

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

/// Quanto tempo leva a princesa a passar de pedir ajuda a ficar contente, em segundos.
const CROSSFADE: f32 = 0.1;
/// O tamanho da jaula, em metros.
const GOAL_SIZE: Vec2 = Vec2::new(2.2, 3.0);

/// Uma etiqueta para a jaula: quando o herói lhe toca, a princesa fica livre.
pub(super) struct CGoal;
/// Uma etiqueta para o pedaço do nível, perto da jaula, onde o herói já ouve a princesa.
pub(super) struct CEarshot;
/// Uma etiqueta para a princesa.
struct CPrincess;

/// A jaula com a princesa, e o que é preciso lembrar dela.
#[derive(Debug)]
pub(super) struct Goal {
    /// Onde a princesa está guardada no mundo do jogo.
    princess: Entity,
}

/// Põe no nível a jaula, a princesa lá dentro, e o pedaço de onde o herói a ouve.
pub(super) fn spawn(world: &mut World, assets: &Assets) -> Result<Goal> {
    spawn_cage(world, assets);
    spawn_earshot(world);
    let princess = spawn_princess(world, assets)?;
    Ok(Goal { princess })
}

/// A jaula, em cima do chão, em [`GOAL`].
fn spawn_cage(world: &mut World, assets: &Assets) {
    let texture = &assets.textures.goal;
    let centre = GROUND - GOAL_SIZE.y / 2.0;
    let draw = sized(texture, GOAL_SIZE).depth(depth::GOAL);
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        CGoal,
        CTransform::at(GOAL, centre),
        zone(GOAL_SIZE),
        sprite,
    ));
}

/// O pedaço invisível de onde o herói ouve a princesa: desde [`HELP_DISTANCE`] antes da jaula
/// até ao fim do nível, de cima a baixo.
fn spawn_earshot(world: &mut World) {
    let earshot = Vec2::new(WIDTH - (GOAL - HELP_DISTANCE), VIEW.y);
    world.spawn((
        CActor,
        CEarshot,
        CTransform::at(WIDTH - earshot.x / 2.0, VIEW.y / 2.0),
        zone(earshot),
    ));
}

/// A princesa, de pé dentro da jaula, a pedir ajuda sem parar. Diz onde ficou guardada.
fn spawn_princess(world: &mut World, assets: &Assets) -> Result<Entity> {
    let mut princess = assets.characters.princess.clone();
    // Dá erro se o movimento mexer uma parte que o esqueleto não tem, como no esqueleto que está
    // no lugar de um que falta.
    princess.play(
        &assets.animations.princess.calling,
        Play::new().looping(true),
    )?;
    Ok(world.spawn((
        CActor,
        CPrincess,
        CTransform::at(GOAL, GROUND - ROOT_HEIGHT),
        princess,
    )))
}

/// Quando se ganha o jogo, a princesa fica contente, uma vez só.
pub(super) fn update(
    world: &mut World,
    goal: &Goal,
    assets: &Assets,
    status: CStatus,
) -> Result<()> {
    if status.outcome == Outcome::Won {
        let freed = Play::new().crossfade(CROSSFADE);
        let mut princess = world.get::<&mut Character>(goal.princess)?;
        // Pedir o movimento que já está a tocar não faz nada, por isso pode pedir-se a cada passo.
        princess.play(&assets.animations.princess.freed, freed)?;
    }
    Ok(())
}
