//! O nível do jogo: o fundo, o chão onde o herói anda, e quem e o que lá está.

mod background;
mod coins;
mod enemies;
mod goal;
mod layout;
mod player;
mod shots;
mod terrain;

use olivia::{
    Result, Update,
    ecs::{CBody, ColliderBuilder, Entity, RigidBodyBuilder, World, entity_of},
    input::{Input, Key},
    math::Vec2,
    physics::{self, Pose, Shape},
};

use crate::{
    assets::Assets,
    random::Random,
    status::{CStatus, Outcome},
};

/// Uma etiqueta para tudo o que um jogo novo troca por outro igual, mas acabado de pôr: os maus,
/// as moedas, a jaula, a princesa, o herói, os tiros e as nuvens.
struct CActor;

/// O nível, com o que é preciso lembrar dele entre um passo e o seguinte.
#[derive(Debug)]
pub(crate) struct Level {
    /// Os bocados do fundo, que andam mais devagar do que o resto.
    background: Vec<background::Tile>,
    /// O herói.
    player: player::Player,
    /// A jaula com a princesa.
    goal: goal::Goal,
}

impl Level {
    /// Onde o nível acaba, em metros.
    pub(crate) const WIDTH: f32 = layout::WIDTH;

    /// Põe o nível no mundo do jogo, tal como fica no começo de um jogo.
    pub(crate) fn spawn(world: &mut World, assets: &Assets) -> Result<Self> {
        // O fundo e o chão ficam para sempre: um jogo novo não precisa de os trocar.
        let background = background::spawn(world, &assets.textures.background);
        terrain::spawn(world, &assets.textures.platform);
        // Tudo o resto é trocado em cada jogo novo.
        let (player, goal) = spawn_actors(world, assets)?;

        Ok(Self {
            background,
            player,
            goal,
        })
    }

    /// Começa um jogo novo: tira tudo o que tem a etiqueta [`CActor`] e põe tudo outra vez.
    pub(crate) fn restart(&mut self, world: &mut World, assets: &Assets) -> Result<()> {
        // Primeiro junta numa lista, porque não se pode tirar coisas enquanto se procura.
        let actors: Vec<Entity> = world.query::<Entity>().with::<&CActor>().iter().collect();
        for actor in actors {
            world.despawn(actor)?;
        }
        (self.player, self.goal) = spawn_actors(world, assets)?;
        Ok(())
    }

    /// Joga um passo no nível, e o que acontece pode mudar o placar `status`.
    pub(crate) fn update(
        &self,
        u: &mut Update<'_>,
        assets: &Assets,
        random: &mut Random,
        status: &mut CStatus,
    ) -> Result<()> {
        // Os maus andam e o herói faz o que as teclas mandam.
        enemies::update(u.world, u.physics, u.timer.dt());
        player::update(u, &self.player, assets, random, *status)?;
        // Enquanto se joga, os tiros acertam e o herói encontra o que toca.
        if status.outcome == Outcome::Playing {
            shots::update(u, assets, random)?;
            player::meet(u, &self.player, assets, random, status)?;
        }
        // Só depois de ver o que o herói tocou é que se mostra como cada um ficou, para se ver
        // logo neste passo se ele se magoou ou ganhou.
        player::act(u, &self.player, assets, *status)?;
        player::thin_farts(u)?;
        goal::update(u.world, &self.goal, assets, *status)
    }

    /// Onde está guardado o herói no mundo do jogo.
    pub(crate) const fn hero(&self) -> Entity {
        self.player.entity
    }

    /// Mexe o fundo para a câmara que está em `camera`, mais devagar do que o resto do nível.
    pub(crate) fn scroll(&self, world: &mut World, camera: f32) -> Result<()> {
        background::scroll(world, &self.background, camera)
    }

    /// O placar com que se joga este passo, ou nada se ainda não se puder jogar.
    pub(crate) fn ready_to_play(
        &mut self,
        world: &mut World,
        input: &Input,
        assets: &Assets,
        status: Entity,
    ) -> Result<Option<CStatus>> {
        let current = self.start_over_if_asked(world, input, assets, status)?;
        // O corpo de quem acabou de chegar só aparece no passo seguinte. Até lá, espera-se.
        let has_body = world.satisfies::<&CBody>(self.player.entity);
        Ok(has_body.then_some(current))
    }

    /// Se o jogo já acabou e se carregou no Enter, começa um jogo novo. Dá o placar com que se
    /// fica, guardado em `status`.
    fn start_over_if_asked(
        &mut self,
        world: &mut World,
        input: &Input,
        assets: &Assets,
        status: Entity,
    ) -> Result<CStatus> {
        let current = *world.get::<&CStatus>(status)?;
        if current.outcome == Outcome::Playing || !input.pressed(Key::Enter) {
            return Ok(current);
        }
        // Põe tudo outra vez no sítio e volta ao placar do começo.
        self.restart(world, assets)?;
        let fresh = CStatus::new();
        *world.get::<&mut CStatus>(status)? = fresh;
        Ok(fresh)
    }
}

/// Põe no nível tudo o que um jogo novo troca: os maus, as moedas, a jaula com a princesa, e o
/// herói. Todos levam a etiqueta [`CActor`].
fn spawn_actors(world: &mut World, assets: &Assets) -> Result<(player::Player, goal::Goal)> {
    enemies::spawn(world, &assets.textures);
    coins::spawn(world, &assets.textures.coin);
    let goal = goal::spawn(world, assets)?;
    let player = player::spawn(world, &assets.characters);
    Ok((player, goal))
}

/// Uma caixa que fica parada e que se pode atravessar, como uma porta aberta: serve só para saber
/// quem lá está dentro.
fn zone(size: Vec2) -> ColliderBuilder {
    ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0).sensor(true)
}

/// Uma caixa que se pode atravessar, num corpo que o jogo mexe dizendo-lhe a que velocidade anda.
fn moved(size: Vec2) -> (RigidBodyBuilder, ColliderBuilder) {
    let body = RigidBodyBuilder::kinematic_velocity_based();
    (body, zone(size))
}

/// Tudo o que toca na forma `shape` posta em `position`.
fn touching(physics: &physics::World, shape: &dyn Shape, position: Vec2) -> Vec<Entity> {
    physics
        .overlap_shape(shape, Pose::from_translation(position))
        .into_iter()
        .filter_map(|collider| entity_of(physics, collider))
        .collect()
}
