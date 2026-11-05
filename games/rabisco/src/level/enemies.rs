//! Os maus, que andam para trás e para a frente, cada um no seu caminho.

use olivia::{
    ecs::{CBody, CSprite, CTransform, World},
    math::Vec2,
    physics,
};

use super::{
    CActor,
    layout::{ENEMIES, GROUND},
    moved,
};
use crate::{assets::Textures, depth, sprite::sized};

/// A rapidez com que o mau que flutua sobe e desce.
const SWAY_SPEED: f32 = 2.5;
/// Quanto o mau que flutua sobe e desce, para cima e para baixo de onde começou, em metros.
const SWAY_HEIGHT: f32 = 0.5;

/// Os tipos de maus, cada um com a sua maneira de andar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Comprido e baixinho, rasteja devagar pelo chão.
    Crawl,
    /// Redondo, rebola depressa pelo chão.
    Roll,
    /// Anda pelo ar, a subir e a descer.
    Float,
}

impl Kind {
    /// O tamanho da caixa do mau, em metros.
    const fn size(self) -> Vec2 {
        match self {
            Self::Crawl => Vec2::new(1.5, 0.87),
            Self::Roll => Vec2::new(0.9, 0.9),
            Self::Float => Vec2::new(0.88, 1.2),
        }
    }

    /// A rapidez do mau, em metros por segundo.
    const fn speed(self) -> f32 {
        match self {
            Self::Crawl => 0.7,
            Self::Roll => 1.9,
            Self::Float => 1.0,
        }
    }
}

/// A altura a que um mau anda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Lane {
    /// Pelo chão.
    Ground,
    /// Pelo ar, com o meio do mau a esta altura.
    Air(f32),
}

/// O caminho de um mau, da ponta da esquerda à ponta da direita.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Span {
    /// A ponta da esquerda.
    pub(super) left: f32,
    /// A ponta da direita.
    pub(super) right: f32,
}

/// Para que lado anda um mau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    /// Para a esquerda.
    Left,
    /// Para a direita.
    Right,
}

impl Direction {
    /// O sinal do lado: -1 para a esquerda, que é para onde o x diminui, e 1 para a direita.
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

/// Onde anda um mau de um tipo: o mapa do nível tem um destes para cada mau.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Patrol {
    /// O tipo de mau.
    kind: Kind,
    /// O caminho que ele anda.
    span: Span,
    /// A altura a que anda.
    lane: Lane,
}

impl Patrol {
    /// Um mau do tipo `kind`, a andar de `left` a `right`, à altura `lane`.
    pub(super) const fn new(kind: Kind, left: f32, right: f32, lane: Lane) -> Self {
        Self {
            kind,
            span: Span { left, right },
            lane,
        }
    }

    /// A altura do meio do mau.
    fn y(self) -> f32 {
        match self.lane {
            // Pelo chão, o meio fica meia altura do mau acima do chão.
            Lane::Ground => GROUND - self.kind.size().y / 2.0,
            Lane::Air(y) => y,
        }
    }
}

/// Um mau no seu caminho, e o que ele tem de lembrar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CEnemy {
    /// O tipo de mau.
    kind: Kind,
    /// O caminho que ele anda.
    span: Span,
    /// Para que lado está a andar agora.
    direction: Direction,
    /// O relógio do mau, em segundos: só o que flutua o usa, para subir e descer.
    time: f32,
}

impl CEnemy {
    /// Dá meia-volta se já passou de uma das pontas do caminho. O x do mau é `x`.
    ///
    /// Só dá meia-volta depois de passar a ponta, e entre as pontas continua como estava. Assim
    /// anda sempre para trás e para a frente, sem sair do caminho.
    fn turn_round(&mut self, x: f32) {
        if x < self.span.left {
            self.direction = Direction::Right;
        } else if x > self.span.right {
            self.direction = Direction::Left;
        }
    }

    /// A velocidade do mau: para o lado para onde anda e, se flutuar, a subir ou a descer.
    fn velocity(&self) -> Vec2 {
        let across = self.direction.sign() * self.kind.speed();
        // Para subir e descer devagarinho, como um barco nas ondas, usa-se o cosseno do relógio.
        let sway = match self.kind {
            Kind::Float => (self.time * SWAY_SPEED).cos() * SWAY_SPEED * SWAY_HEIGHT,
            Kind::Crawl | Kind::Roll => 0.0,
        };
        Vec2::new(across, sway)
    }

    /// A rapidez com que o mau que rebola dá voltas: a certa para rolar sem escorregar, como uma
    /// roda de bicicleta. Para a esquerda, roda ao contrário.
    fn spin(&self) -> f32 {
        let radius = self.kind.size().x / 2.0;
        self.direction.sign() * self.kind.speed() / radius
    }

    /// Vira o desenho `sprite` do mau para o lado para onde ele anda, como num espelho.
    fn face(&self, sprite: &mut CSprite) {
        let scale = self.kind.size() / sprite.texture.size();
        sprite.draw = sprite
            .draw
            .scale(scale * Vec2::new(self.direction.sign(), 1.0));
    }
}

/// Põe no nível todos os maus, cada um na ponta da direita do seu caminho, a andar para a esquerda.
pub(super) fn spawn(world: &mut World, textures: &Textures) {
    for patrol in ENEMIES {
        spawn_enemy(world, textures, patrol);
    }
}

/// Põe no nível o mau que anda como diz `patrol`.
fn spawn_enemy(world: &mut World, textures: &Textures, patrol: Patrol) {
    let Patrol { kind, span, .. } = patrol;
    // Escolhe o desenho do tipo de mau.
    let texture = match kind {
        Kind::Crawl => &textures.crawl,
        Kind::Roll => &textures.roll,
        Kind::Float => &textures.float,
    };
    let draw = sized(texture, kind.size()).depth(depth::THING);
    let enemy = CEnemy {
        kind,
        span,
        direction: Direction::Left,
        time: 0.0,
    };
    // Dá-lhe um corpo que o jogo mexe, e põe-no na ponta da direita.
    let (body, collider) = moved(kind.size());
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        enemy,
        CTransform::at(span.right, patrol.y()),
        body,
        collider,
        sprite,
    ));
}

/// Mexe todos os maus durante um passo de `dt` segundos, para trás e para a frente no seu caminho.
pub(super) fn update(world: &mut World, physics: &mut physics::World, dt: f32) {
    for (CBody(body), transform, enemy, sprite) in
        world.query_mut::<(&CBody, &CTransform, &mut CEnemy, &mut CSprite)>()
    {
        // O relógio do mau anda.
        enemy.time += dt;
        // Dá meia-volta, se já chegou a uma ponta.
        enemy.turn_round(transform.position.x);
        // O corpo dos maus não cai nem é empurrado: o jogo diz-lhe a velocidade em cada passo.
        let body = physics.body_mut(*body);
        body.set_linvel(enemy.velocity(), true);
        // O que rebola dá voltas, e o que rasteja, o único com frente e trás, vira-se.
        match enemy.kind {
            Kind::Roll => body.set_angvel(enemy.spin(), true),
            Kind::Crawl => enemy.face(sprite),
            Kind::Float => {}
        }
    }
}
