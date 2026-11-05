//! Os tiros que o herói dispara contra os maus.

use olivia::{
    Result, Update,
    ecs::{CSprite, CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World},
    math::Vec2,
    physics::Ball,
};

use super::{CActor, enemies::CEnemy, terrain::CSolid, touching};
use crate::{assets::Assets, depth, random::Random, sprite::sized};

/// A rapidez com que o tiro sai, em metros por segundo.
const SHOT_SPEED: f32 = 7.2;
/// Quanto a gravidade puxa o tiro: um bocadinho, para ele fazer uma curva e cair em cima de quem
/// rasteja pelo chão.
const SHOT_GRAVITY: f32 = 0.4;
/// O tamanho do tiro, em metros.
const SHOT_SIZE: f32 = 0.48;
/// Quanto tempo o tiro voa antes de desaparecer, em segundos.
const SHOT_TIME: f32 = 1.2;

/// Um tiro a voar, e há quanto tempo voa.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CShot {
    /// Há quanto tempo voa, em segundos.
    time: f32,
}

/// Um tiro neste passo: onde está guardado, onde está, e há quanto tempo voa.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Flight {
    /// Onde o tiro está guardado no mundo do jogo.
    entity: Entity,
    /// Onde o tiro está.
    position: Vec2,
    /// Há quanto tempo voa, em segundos.
    time: f32,
}

/// Dispara um tiro a partir de `centre`: para a direita se `direction` for 1, para a esquerda se
/// for -1.
pub(super) fn spawn(world: &mut World, assets: &Assets, centre: Vec2, direction: f32) {
    // O desenho vira-se como o herói, para apontar para onde voa.
    let texture = &assets.textures.projectile;
    let draw = sized(texture, Vec2::splat(SHOT_SIZE));
    let draw = draw
        .scale(draw.scale_value() * Vec2::new(direction, 1.0))
        .depth(depth::THING);
    // Sai a voar a direito, e a gravidade vai-o puxando para baixo devagarinho.
    let body = RigidBodyBuilder::dynamic()
        .gravity_scale(SHOT_GRAVITY)
        .linvel(Vec2::new(direction * SHOT_SPEED, 0.0));
    world.spawn((
        CActor,
        CShot { time: 0.0 },
        CTransform::at(centre.x, centre.y),
        body,
        ColliderBuilder::ball(SHOT_SIZE / 2.0).sensor(true),
        CSprite::new(texture.clone()).draw(draw),
    ));
}

/// Faz voar todos os tiros durante um passo: em que acertam, e se desaparecem.
pub(super) fn update(u: &mut Update<'_>, assets: &Assets, random: &mut Random) -> Result<()> {
    for flight in fly(u.world, u.timer.dt()) {
        land(u, assets, random, flight)?;
    }
    Ok(())
}

/// Faz andar o relógio de cada tiro `dt` segundos, e junta-os todos numa lista.
///
/// Junta-os primeiro numa lista porque os tiros que acertam vão desaparecer, e não se pode tirar
/// coisas do mundo enquanto se está a procurar nele.
fn fly(world: &mut World, dt: f32) -> Vec<Flight> {
    world
        .query_mut::<(Entity, &CTransform, &mut CShot)>()
        .into_iter()
        .map(|(entity, transform, shot)| {
            shot.time += dt;
            Flight {
                entity,
                position: transform.position,
                time: shot.time,
            }
        })
        .collect()
}

/// O que acontece neste passo ao tiro `flight`.
fn land(u: &mut Update<'_>, assets: &Assets, random: &mut Random, flight: Flight) -> Result<()> {
    let touched = touching(u.physics, &Ball::new(SHOT_SIZE / 2.0), flight.position);
    // Um mau em que o tiro toque desaparece.
    let enemy = touched.iter().find(|e| u.world.satisfies::<&CEnemy>(**e));
    if let Some(enemy) = enemy {
        u.world.despawn(*enemy)?;
        u.audio.play(assets.sounds.hit.pick(random));
    }
    // O tiro desaparece se acertou num mau, se bateu numa coisa que não se atravessa, ou se já
    // voou tempo de mais.
    let is_stopped = touched.iter().any(|e| u.world.satisfies::<&CSolid>(*e));
    if enemy.is_some() || is_stopped || flight.time > SHOT_TIME {
        u.world.despawn(flight.entity)?;
    }
    Ok(())
}
