//! O chão e as plataformas, as paredes nas pontas e o buraco por baixo de tudo.

use olivia::{
    ecs::{CSprite, CTransform, ColliderBuilder, World},
    graphics::{Draw, Texture},
    math::{Rect, Vec2},
};

use super::{
    layout::{PIT, PLATFORMS, WIDTH},
    zone,
};
use crate::{camera::VIEW, depth};

/// O bocado de plataforma mais estreito que vale a pena desenhar, em metros: menos do que isto é
/// só um erro das contas.
const MIN_TILE: f32 = 1e-3;

/// Uma etiqueta para o que não se pode atravessar: nem o herói, nem os tiros.
pub(super) struct CSolid;
/// Uma etiqueta para o buraco onde o herói cai.
pub(super) struct CPit;

/// Põe no mundo do jogo as plataformas, as paredes nas pontas e o buraco por baixo de tudo.
pub(super) fn spawn(world: &mut World, platform: &Texture) {
    for rect in PLATFORMS {
        spawn_platform_body(world, rect);
        spawn_platform_tiles(world, platform, rect);
    }
    spawn_walls(world);
    spawn_pit(world);
}

/// A parte da plataforma `rect` que não se deixa atravessar: uma caixa invisível.
fn spawn_platform_body(world: &mut World, rect: Rect) {
    let centre = rect.pos + rect.size / 2.0;
    world.spawn((
        CSolid,
        CTransform::at(centre.x, centre.y),
        // Sem atrito, para o herói não ficar preso às bordas.
        ColliderBuilder::cuboid(rect.size.x / 2.0, rect.size.y / 2.0).friction(0.0),
    ));
}

/// O desenho da plataforma `rect`: o desenho `platform` posto lado a lado, e o último bocado
/// cortado para caber, como os azulejos numa parede.
fn spawn_platform_tiles(world: &mut World, platform: &Texture, rect: Rect) {
    // Cada bocado fica da altura da plataforma, e com a largura certa para não ficar esticado.
    let scale = rect.size.y / platform.size().y;
    let tile = platform.size().x * scale;
    let end = rect.pos.x + rect.size.x;
    let mut x = rect.pos.x;
    while end - x > MIN_TILE {
        // O último bocado só usa a parte do desenho que cabe até ao fim da plataforma.
        let width = (end - x).min(tile);
        let cut = platform.region(Rect::new(0.0, 0.0, width / scale, platform.size().y));
        let draw = Draw::new().scale(Vec2::splat(scale)).depth(depth::PLATFORM);
        world.spawn((CTransform::at(x, rect.pos.y), CSprite::new(cut).draw(draw)));
        x += tile;
    }
}

/// As paredes invisíveis nas pontas do nível, para ninguém sair por lá.
fn spawn_walls(world: &mut World) {
    for x in [-0.5, WIDTH + 0.5] {
        let wall = ColliderBuilder::cuboid(0.5, VIEW.y * 2.0).friction(0.0);
        world.spawn((CSolid, CTransform::at(x, VIEW.y / 2.0), wall));
    }
}

/// O buraco: uma caixa invisível ao longo de todo o nível, a começar em [`PIT`] e para baixo.
fn spawn_pit(world: &mut World) {
    let pit = zone(Vec2::new(WIDTH, 1.0));
    world.spawn((CPit, CTransform::at(WIDTH / 2.0, PIT + 0.5), pit));
}
