//! Onde está cada coisa no nível, como num mapa.
//!
//! As medidas são em metros. O x cresce para a direita e o y cresce para baixo: quanto maior o y,
//! mais abaixo fica a coisa.

use olivia::math::{Rect, Vec2};

use super::enemies::{Kind, Lane, Patrol};

/// A altura de cima do chão.
pub(super) const GROUND: f32 = 6.0;
/// Onde o nível acaba.
pub(super) const WIDTH: f32 = 44.0;
/// A altura a partir da qual uma queda custa uma vida.
pub(super) const PIT: f32 = 8.4;

/// As plataformas onde o herói anda: cada uma com o x e o y do canto de cima à esquerda, a
/// largura e a altura.
///
/// As primeiras quatro são o chão, com buracos entre elas. As outras estão no ar: deixam espaço
/// para a cabeça do herói passar por baixo, e as mais altas só se alcançam com o salto extra.
pub(super) const PLATFORMS: [Rect; 10] = [
    Rect::new(0.0, GROUND, 13.0, 1.2),
    Rect::new(15.0, GROUND, 9.0, 1.2),
    Rect::new(25.8, GROUND, 5.0, 1.2),
    Rect::new(32.6, GROUND, 11.4, 1.2),
    Rect::new(6.4, 3.8, 2.8, 0.34),
    Rect::new(10.2, 2.6, 2.6, 0.34),
    Rect::new(17.2, 3.8, 3.0, 0.34),
    Rect::new(21.4, 2.6, 2.6, 0.34),
    Rect::new(27.0, 3.8, 2.4, 0.34),
    Rect::new(34.4, 3.8, 3.0, 0.34),
];

/// O x da jaula onde a princesa está presa, em cima do chão.
pub(super) const GOAL: f32 = 41.5;
/// A que distância da jaula o herói tem de chegar para ouvir a princesa pedir ajuda.
pub(super) const HELP_DISTANCE: f32 = 5.2;

/// Os sítios no chão onde o herói volta a começar depois de cair: o último por onde passou.
pub(super) const CHECKPOINTS: [f32; 4] = [1.2, 15.6, 26.2, 33.0];

/// Onde estão as moedas.
pub(super) const COINS: [Vec2; 14] = [
    Vec2::new(3.0, 5.4),
    Vec2::new(4.2, 5.4),
    Vec2::new(7.2, 3.2),
    Vec2::new(8.4, 3.2),
    Vec2::new(11.5, 2.0),
    Vec2::new(14.0, 4.2),
    Vec2::new(18.0, 3.2),
    Vec2::new(19.4, 3.2),
    Vec2::new(22.7, 2.0),
    Vec2::new(24.9, 4.2),
    Vec2::new(28.2, 3.2),
    Vec2::new(31.7, 4.2),
    Vec2::new(35.2, 3.2),
    Vec2::new(36.6, 3.2),
];

/// Os maus: de que tipo é cada um, entre que pontas anda, e a que altura.
pub(super) const ENEMIES: [Patrol; 7] = [
    Patrol::new(Kind::Crawl, 5.0, 12.0, Lane::Ground),
    Patrol::new(Kind::Float, 10.4, 14.6, Lane::Air(4.0)),
    Patrol::new(Kind::Roll, 16.0, 23.4, Lane::Ground),
    Patrol::new(Kind::Crawl, 19.5, 23.5, Lane::Ground),
    Patrol::new(Kind::Float, 26.4, 32.0, Lane::Air(4.0)),
    Patrol::new(Kind::Roll, 33.2, 38.0, Lane::Ground),
    Patrol::new(Kind::Crawl, 35.0, 38.8, Lane::Ground),
];
