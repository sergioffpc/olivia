//! O herói: a personagem que tu controlas com o teclado.
//!
//! As setas ← e → fazem-no correr, o Espaço ou a seta ↑ fazem-no saltar, e o X faz-o disparar.

use olivia::{
    Result, Update,
    animation::{Animation, Character, Play},
    ecs::{
        CBody, CSprite, CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World, entity_of,
    },
    graphics::Draw,
    input::{Input, Key},
    math::{Color, Vec2},
    physics::{self, Cuboid, LockedAxes},
};

use super::{
    CActor,
    coins::CCoin,
    enemies::CEnemy,
    goal::{CEarshot, CGoal},
    layout::{CHECKPOINTS, GROUND},
    shots,
    terrain::{CPit, CSolid},
    touching,
};
use crate::{
    assets::{Assets, Characters, HeroAnimations, PART_SCALE, ROOT_HEIGHT},
    depth,
    random::Random,
    sprite::sized,
    status::{CStatus, Outcome},
};

/// O tamanho da caixa do herói, em metros: é com ela que ele toca nas coisas.
const HERO_SIZE: Vec2 = Vec2::new(0.7, 1.6);
/// A rapidez com que o herói corre, em metros por segundo.
const RUN_SPEED: f32 = 3.4;
/// Quanto mais a gravidade puxa o herói do que o resto: o salto chega alto, mas é rápido, para
/// não parecer que ele está na Lua.
const HERO_GRAVITY: f32 = 2.5;
/// A rapidez com que o herói sai do chão ao saltar, em metros por segundo.
///
/// Sobe uns 2,6 metros: passa as plataformas baixas, que estão 2,2 metros acima do chão, mas não
/// chega às altas, que estão 3,4 metros acima dele.
const JUMP_SPEED: f32 = 11.3;
/// A rapidez com que o herói sobe no salto extra, já no ar, em metros por segundo: mais uns 1,3
/// metros, que chegam para as plataformas altas.
const EXTRA_JUMP_SPEED: f32 = 8.0;
/// Quanto tempo o herói anda às voltas no salto extra, em segundos.
const SPIN: f32 = 0.6;
/// Até que distância debaixo dos pés ainda conta como estar de pé em alguma coisa, em metros.
const FOOTING: f32 = 0.08;
/// O tempo mínimo entre dois tiros, em segundos: mais do que o movimento de disparar demora.
const SHOT_COOLDOWN: f32 = 0.45;
/// Quanto tempo o braço leva a levantar-se antes de o tiro sair, em segundos.
const SHOT_DELAY: f32 = 0.16;
/// Quanto tempo leva o braço a entrar e a sair, em segundos.
const OVERLAY_FADE: f32 = 0.06;
/// A que distância da mão aparece o tiro, ao longo do lançador, em metros.
const MUZZLE: f32 = 0.6;
/// O tamanho da nuvem que sai no salto extra, em metros.
const FART_SIZE: f32 = 0.75;
/// Quanto tempo dura a nuvem, em segundos.
const FART_TIME: f32 = 0.9;
/// Quanto tempo leva um movimento a dar lugar ao seguinte, em segundos.
const CROSSFADE: f32 = 0.1;
/// Quantas vezes por segundo o herói pisca enquanto nada o pode magoar.
const BLINKS_PER_SECOND: f64 = 8.0;
/// Quanto tempo o herói mostra que está magoado, em segundos.
const HURT: f32 = 0.7;
/// Quanto tempo nada pode magoar o herói depois de ele se magoar, em segundos. Assim, tocar num
/// mau custa uma vida só, e não todas de uma vez.
const INVULNERABLE: f32 = 1.6;

/// A nuvem que sai no salto extra, e quanto tempo lhe falta, em segundos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CFart(pub(super) f32);

/// O que o herói tem de lembrar entre um passo e o seguinte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CHero {
    /// Se está a olhar para a esquerda.
    is_facing_left: bool,
    /// Se está de pé em alguma coisa.
    is_grounded: bool,
    /// Se ainda pode dar o salto extra no ar.
    has_extra_jump: bool,
    /// Quanto tempo falta para acabar de dar voltas no salto extra.
    spin: f32,
    /// Quanto tempo falta para deixar de mostrar que está magoado.
    hurt: f32,
    /// Quanto tempo falta para algo o poder magoar outra vez.
    invulnerable: f32,
    /// Quanto tempo falta para poder disparar outra vez.
    cooldown: f32,
    /// Quanto tempo falta para o tiro sair, se o braço já se está a levantar.
    pending_shot: Option<f32>,
}

impl CHero {
    /// Um herói acabado de chegar: a olhar para a direita, no ar, com o salto extra.
    const fn new() -> Self {
        Self {
            is_facing_left: false,
            is_grounded: false,
            has_extra_jump: true,
            spin: 0.0,
            hurt: 0.0,
            invulnerable: 0.0,
            cooldown: 0.0,
            pending_shot: None,
        }
    }

    /// Faz andar para trás os relógios do herói `dt` segundos, até chegarem a zero.
    fn count_down(&mut self, dt: f32) {
        self.spin = (self.spin - dt).max(0.0);
        self.hurt = (self.hurt - dt).max(0.0);
        self.invulnerable = (self.invulnerable - dt).max(0.0);
        self.cooldown = (self.cooldown - dt).max(0.0);
    }

    /// Vira o herói para o lado para onde corre a `speed`. Parado, fica a olhar para onde ia.
    fn face(&mut self, speed: f32) {
        if speed != 0.0 {
            self.is_facing_left = speed < 0.0;
        }
    }

    /// Lembra se o herói está de pé: tem de ter alguma coisa debaixo dos pés (`is_standing`) e
    /// não estar a subir (a velocidade para baixo `fall` não é negativa). De pé, ganha outra vez o
    /// salto extra.
    fn stand(&mut self, is_standing: bool, fall: f32) {
        // A subir, a passar pela borda de uma plataforma, ainda não está em cima dela.
        self.is_grounded = is_standing && fall >= 0.0;
        if self.is_grounded {
            self.has_extra_jump = true;
        }
    }
}

/// As teclas que o jogador carregou neste passo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // cada tecla está carregada ou não, e só isso
struct Keys {
    /// A seta ← está carregada.
    left: bool,
    /// A seta → está carregada.
    right: bool,
    /// Carregou-se agora para saltar.
    jump: bool,
    /// Carregou-se agora para disparar.
    shoot: bool,
}

impl Keys {
    /// Lê as teclas. Quando o jogo já acabou, é como se nenhuma estivesse carregada.
    fn read(input: &Input, status: CStatus) -> Self {
        if status.outcome != Outcome::Playing {
            return Self::default();
        }
        Self {
            left: input.down(Key::Left),
            right: input.down(Key::Right),
            jump: input.pressed(Key::Space) || input.pressed(Key::Up),
            shoot: input.pressed(Key::X),
        }
    }

    /// A rapidez com que o herói corre: só corre com uma seta carregada. Com as duas, ou nenhuma,
    /// para logo.
    const fn run_speed(self) -> f32 {
        match (self.left, self.right) {
            (true, false) => -RUN_SPEED,
            (false, true) => RUN_SPEED,
            _ => 0.0,
        }
    }
}

/// Onde o herói está guardado no mundo do jogo.
#[derive(Debug)]
pub(super) struct Player {
    /// Onde o herói está guardado no mundo do jogo.
    pub(super) entity: Entity,
}

/// Põe o herói no nível, de pé no chão, no primeiro sítio de recomeço.
pub(super) fn spawn(world: &mut World, characters: &Characters) -> Player {
    // Um corpo que cai com a gravidade, mas que não tomba, e que nunca adormece.
    let body = RigidBodyBuilder::dynamic()
        .gravity_scale(HERO_GRAVITY)
        .locked_axes(LockedAxes::ROTATION_LOCKED)
        .can_sleep(false);
    // O herói segura-se pelo meio do tronco, e a caixa tem de acabar nos pés.
    let centre = ROOT_HEIGHT - HERO_SIZE.y / 2.0;
    let collider = ColliderBuilder::cuboid(HERO_SIZE.x / 2.0, HERO_SIZE.y / 2.0)
        .translation(Vec2::new(0.0, centre))
        .friction(0.0);
    let entity = world.spawn((
        CActor,
        CHero::new(),
        CTransform::at(CHECKPOINTS[0], GROUND - ROOT_HEIGHT),
        body,
        collider,
        characters.hero.clone(),
    ));
    Player { entity }
}

/// Faz o herói correr, saltar e disparar durante um passo, como as teclas mandam.
pub(super) fn update(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: CStatus,
) -> Result<()> {
    let dt = u.timer.dt();
    let keys = Keys::read(u.input, status);
    // Vai buscar o herói ao mundo do jogo.
    let root = u.world.get::<&CTransform>(player.entity)?.position;
    let mut hero = *u.world.get::<&CHero>(player.entity)?;
    // Os relógios do herói andam para trás.
    hero.count_down(dt);
    // Corre e salta.
    move_hero(u, player, assets, random, &mut hero, keys)?;
    // Levanta o braço, se se carregou para disparar, e dispara quando ele estiver no ar.
    if keys.shoot {
        raise_arm(u, player, assets, random, &mut hero)?;
    }
    release_shot(u.world, player, assets, &mut hero, root, dt)?;
    // Guarda no mundo do jogo como o herói ficou.
    *u.world.get::<&mut CHero>(player.entity)? = hero;
    Ok(())
}

/// Faz o herói correr e saltar, mudando a velocidade do seu corpo.
fn move_hero(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    hero: &mut CHero,
    keys: Keys,
) -> Result<()> {
    let feet = u.world.get::<&CTransform>(player.entity)?.position + Vec2::new(0.0, ROOT_HEIGHT);
    let body = u.world.get::<&CBody>(player.entity)?.0;
    let mut velocity = u.physics.body(body).linvel();
    // Corre para o lado da seta carregada, e vira-se para lá.
    velocity.x = keys.run_speed();
    hero.face(velocity.x);
    // Vê se está de pé em alguma coisa.
    hero.stand(is_standing(u.physics, u.world, feet), velocity.y);
    // Salta, se se carregou para saltar.
    if keys.jump {
        jump(u, assets, random, hero, &mut velocity, feet);
    }
    u.physics.body_mut(body).set_linvel(velocity, true);
    Ok(())
}

/// Se há alguma coisa que não se atravessa debaixo de um dos lados dos pés, que estão em `feet`.
fn is_standing(physics: &physics::World, world: &World, feet: Vec2) -> bool {
    // Olha para baixo a partir dos dois lados dos pés, como quem apalpa o chão com uma bengala.
    [-0.3, 0.3].iter().any(|side| {
        // Começa logo abaixo da caixa, senão encontrava o próprio herói.
        let start = feet + Vec2::new(*side, 0.02);
        physics
            .ray_cast(start, Vec2::Y, FOOTING)
            .and_then(|(collider, _)| entity_of(physics, collider))
            .is_some_and(|entity| world.satisfies::<&CSolid>(entity))
    })
}

/// Salta do chão ou, se já estiver no ar, dá o salto extra, se ainda o tiver. A velocidade do
/// herói é `velocity` e os pés estão em `feet`.
fn jump(
    u: &mut Update<'_>,
    assets: &Assets,
    random: &mut Random,
    hero: &mut CHero,
    velocity: &mut Vec2,
    feet: Vec2,
) {
    if hero.is_grounded {
        // Do chão: para cima, que é para onde o y diminui.
        velocity.y = -JUMP_SPEED;
        hero.is_grounded = false;
        u.audio.play(assets.sounds.hero.jump.pick(random));
    } else if hero.has_extra_jump {
        // No ar: o salto extra só se dá uma vez, e o herói dá voltas e deixa uma nuvem.
        velocity.y = -EXTRA_JUMP_SPEED;
        hero.has_extra_jump = false;
        hero.spin = SPIN;
        u.audio.play(assets.sounds.hero.fart.pick(random));
        spawn_fart(u.world, assets, feet);
    }
}

/// Põe a nuvem do salto extra nos pés do herói, que estão em `feet`.
fn spawn_fart(world: &mut World, assets: &Assets, feet: Vec2) {
    let texture = &assets.textures.fart;
    let draw = sized(texture, Vec2::splat(FART_SIZE)).depth(depth::THING);
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        CFart(FART_TIME),
        CTransform::at(feet.x, feet.y),
        sprite,
    ));
}

/// Levanta o braço para disparar, se já passou tempo bastante desde o último tiro.
fn raise_arm(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    hero: &mut CHero,
) -> Result<()> {
    if hero.cooldown > 0.0 {
        return Ok(());
    }
    hero.cooldown = SHOT_COOLDOWN;
    hero.pending_shot = Some(SHOT_DELAY);
    u.audio.play(assets.sounds.hero.shoot.pick(random));
    // O braço levanta-se por cima do que o herói estiver a fazer: pode correr e disparar.
    let overlay = Play::new().crossfade(OVERLAY_FADE);
    let mut character = u.world.get::<&mut Character>(player.entity)?;
    character.overlay(&assets.animations.hero.shoot, overlay)?;
    Ok(())
}

/// Conta o tempo que o braço leva a levantar-se, `dt` segundos de cada vez, e dispara quando
/// estiver no ar. O herói segura-se em `root`.
fn release_shot(
    world: &mut World,
    player: &Player,
    assets: &Assets,
    hero: &mut CHero,
    root: Vec2,
    dt: f32,
) -> Result<()> {
    hero.pending_shot = hero.pending_shot.map(|left| left - dt);
    if hero.pending_shot.is_some_and(|left| left <= 0.0) {
        hero.pending_shot = None;
        fire(world, player, assets, hero, root)?;
    }
    Ok(())
}

/// Dispara um tiro do lançador, a direito para onde o herói olha. O herói segura-se em `root`.
fn fire(
    world: &mut World,
    player: &Player,
    assets: &Assets,
    hero: &CHero,
    root: Vec2,
) -> Result<()> {
    // O tiro sai um pouco à frente da mão que segura o lançador.
    let hand = world.get::<&Character>(player.entity)?.joint("hand_right");
    let hand = hand.ok_or("the hero has no hand_right")?;
    let direction = if hero.is_facing_left { -1.0 } else { 1.0 };
    let centre = root + hand.offset + Vec2::new(direction * MUZZLE, 0.0);
    shots::spawn(world, assets, centre, direction);
    Ok(())
}

/// O que a caixa do herói está a tocar neste passo.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // cada coisa toca-se ou não, e só isso
struct Touches {
    /// As moedas em que toca.
    coins: Vec<Entity>,
    /// Se toca num mau.
    enemy: bool,
    /// Se caiu no buraco.
    pit: bool,
    /// Se chegou à jaula.
    cage: bool,
    /// Se está perto da jaula, onde se ouve a princesa.
    earshot: bool,
}

impl Touches {
    /// Vê tudo o que a caixa do herói, que se segura em `root`, está a tocar.
    fn find(world: &World, physics: &physics::World, root: Vec2) -> Self {
        let centre = root + Vec2::new(0.0, ROOT_HEIGHT - HERO_SIZE.y / 2.0);
        let area = Cuboid::new(HERO_SIZE / 2.0);
        let mut touches = Self::default();
        // Vê a etiqueta de cada coisa em que toca, para saber o que é.
        for entity in touching(physics, &area, centre) {
            if world.satisfies::<&CCoin>(entity) {
                touches.coins.push(entity);
            }
            touches.enemy |= world.satisfies::<&CEnemy>(entity);
            touches.pit |= world.satisfies::<&CPit>(entity);
            touches.cage |= world.satisfies::<&CGoal>(entity);
            touches.earshot |= world.satisfies::<&CEarshot>(entity);
        }
        touches
    }
}

/// O que acontece ao herói por causa do que ele toca: apanha moedas, magoa-se nos maus, cai no
/// buraco, ouve a princesa e, na jaula, ganha o jogo. Tudo isto muda o placar `status`.
pub(super) fn meet(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    let root = u.world.get::<&CTransform>(player.entity)?.position;
    let touches = Touches::find(u.world, u.physics, root);
    for coin in touches.coins {
        pick_up(u, assets, random, status, coin)?;
    }
    if touches.earshot {
        hear_princess(u, assets, random, status);
    }
    // Cair custa sempre uma vida. Tocar num mau só custa se nada o puder magoar.
    if touches.pit {
        lose_life(u, player, assets, random, status)?;
        back_to_checkpoint(u, player, root.x)?;
    } else if touches.enemy {
        get_hurt(u, player, assets, random, status)?;
    }
    if touches.cage {
        win(u, assets, random, status)?;
    }
    Ok(())
}

/// Apanha a moeda `coin`: ela desaparece e o herói ganha um ponto.
fn pick_up(
    u: &mut Update<'_>,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
    coin: Entity,
) -> Result<()> {
    u.world.despawn(coin)?;
    status.score += 1;
    u.audio.play(assets.sounds.pickup.pick(random));
    Ok(())
}

/// A princesa pede ajuda, mas só da primeira vez que o herói se aproxima.
fn hear_princess(u: &mut Update<'_>, assets: &Assets, random: &mut Random, status: &mut CStatus) {
    if !status.has_called {
        status.has_called = true;
        u.audio.play(assets.sounds.princess.help.pick(random));
    }
}

/// Põe o herói outra vez de pé no chão, parado, no último sítio de recomeço por onde passou. Ele
/// caiu em `x`.
fn back_to_checkpoint(u: &mut Update<'_>, player: &Player, x: f32) -> Result<()> {
    // O último sítio de recomeço que fica antes de `x`, ou o primeiro se não houver nenhum.
    let reached = CHECKPOINTS
        .iter()
        .rev()
        .find(|checkpoint| **checkpoint <= x);
    let x = *reached.unwrap_or(&CHECKPOINTS[0]);
    let body = u.world.get::<&CBody>(player.entity)?.0;
    let body = u.physics.body_mut(body);
    body.set_translation(Vec2::new(x, GROUND - ROOT_HEIGHT), true);
    body.set_linvel(Vec2::ZERO, true);
    Ok(())
}

/// O herói magoa-se num mau e perde uma vida, se nada o puder magoar neste momento.
fn get_hurt(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    let invulnerable = u.world.get::<&CHero>(player.entity)?.invulnerable;
    if invulnerable > 0.0 {
        return Ok(());
    }
    lose_life(u, player, assets, random, status)
}

/// Tira uma vida ao herói e magoa-o por um bocadinho. Na última vida, o jogo acaba.
fn lose_life(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    status.lives -= 1;
    // Durante um bocadinho mostra que se magoou, e nada o pode magoar outra vez.
    let mut hero = u.world.get::<&mut CHero>(player.entity)?;
    hero.hurt = HURT;
    hero.invulnerable = INVULNERABLE;
    drop(hero);
    if status.lives == 0 {
        status.outcome = Outcome::Lost;
        u.audio.play(assets.sounds.gameover.pick(random));
    } else {
        u.audio.play(assets.sounds.hero.hurt.pick(random));
    }
    Ok(())
}

/// O herói chegou à jaula e ganha o jogo: a jaula desaparece e a princesa fica livre.
fn win(
    u: &mut Update<'_>,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    // Se perdeu a última vida neste mesmo passo, já não ganha.
    if status.outcome != Outcome::Playing {
        return Ok(());
    }
    status.outcome = Outcome::Won;
    u.audio.play(assets.sounds.hero.win.pick(random));
    // Junta primeiro numa lista, porque não se pode tirar coisas enquanto se procura.
    let cage: Vec<Entity> = u.world.query::<Entity>().with::<&CGoal>().iter().collect();
    for entity in cage {
        u.world.despawn(entity)?;
    }
    Ok(())
}

/// Mostra o herói como ele está: com o movimento do que está a fazer, virado para onde olha, e a
/// piscar enquanto nada o pode magoar.
pub(super) fn act(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    status: CStatus,
) -> Result<()> {
    let hero = *u.world.get::<&CHero>(player.entity)?;
    let body = u.world.get::<&CBody>(player.entity)?.0;
    let is_moving = u.physics.body(body).linvel().x != 0.0;
    let is_blinking = is_blinking(&hero, status, u.timer.elapsed());
    let mut character = u.world.get::<&mut Character>(player.entity)?;
    animate(
        &mut character,
        &assets.animations.hero,
        &hero,
        status,
        is_moving,
    )?;
    character.draw = look(character.draw, &hero, is_blinking);
    Ok(())
}

/// Põe o herói, `character`, a fazer o movimento certo, e baixa-lhe o braço depois de disparar.
fn animate(
    character: &mut Character,
    moves: &HeroAnimations,
    hero: &CHero,
    status: CStatus,
    is_moving: bool,
) -> Result<()> {
    let (chosen, looping) = choose_move(moves, hero, status, is_moving);
    // Pedir o movimento que já está a tocar não faz nada, por isso pode pedir-se a cada passo.
    character.play(chosen, Play::new().looping(looping).crossfade(CROSSFADE))?;
    if character.finished(&moves.shoot) {
        character.remove_overlay(&moves.shoot, OVERLAY_FADE);
    }
    Ok(())
}

/// O movimento que o herói deve fazer, e se se repete sem parar.
///
/// Ganha o primeiro que servir: como o jogo acabou, depois estar magoado, as voltas do salto
/// extra, estar no ar, e por fim andar ou estar parado, os únicos que se repetem.
fn choose_move<'a>(
    moves: &'a HeroAnimations,
    hero: &CHero,
    status: CStatus,
    is_moving: bool,
) -> (&'a Animation, bool) {
    if status.outcome == Outcome::Won {
        (&moves.win, false)
    } else if status.outcome == Outcome::Lost || hero.hurt > 0.0 {
        (&moves.hurt, false)
    } else if hero.spin > 0.0 {
        (&moves.double_jump, false)
    } else if !hero.is_grounded {
        (&moves.jump, false)
    } else if is_moving {
        (&moves.walk, true)
    } else {
        (&moves.idle, true)
    }
}

/// Se o herói deve estar escondido neste momento, para piscar enquanto nada o pode magoar.
///
/// Conta o tempo de jogo, `elapsed`, em bocadinhos: nos pares esconde-se, nos ímpares aparece.
fn is_blinking(hero: &CHero, status: CStatus, elapsed: f64) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // o tempo nunca é negativo
    let tick = (elapsed * BLINKS_PER_SECOND) as u64;
    hero.invulnerable > 0.0 && status.outcome == Outcome::Playing && tick.is_multiple_of(2)
}

/// Como desenhar o herói, a partir de `draw`: virado para onde olha, e escondido se estiver a
/// piscar.
fn look(draw: Draw, hero: &CHero, is_blinking: bool) -> Draw {
    // Para olhar para a esquerda, o desenho vira-se ao contrário, como num espelho.
    let scale = if hero.is_facing_left {
        Vec2::new(-PART_SCALE, PART_SCALE)
    } else {
        Vec2::splat(PART_SCALE)
    };
    let alpha = if is_blinking { 0.0 } else { 1.0 };
    draw.scale(scale).color(Color::new(1.0, 1.0, 1.0, alpha))
}

/// Faz crescer as nuvens do salto extra até ao dobro do tamanho, cada vez mais transparentes, até
/// desaparecerem.
pub(super) fn thin_farts(u: &mut Update<'_>) -> Result<()> {
    let dt = u.timer.dt();
    // Junta primeiro numa lista as que acabaram, porque não se pode tirar coisas enquanto se
    // procura.
    let mut gone = Vec::new();
    for (entity, fart, sprite) in u.world.query_mut::<(Entity, &mut CFart, &mut CSprite)>() {
        if fade(fart, sprite, dt) {
            gone.push(entity);
        }
    }
    for entity in gone {
        u.world.despawn(entity)?;
    }
    Ok(())
}

/// Faz andar o relógio da nuvem `fart` `dt` segundos, e muda o desenho `sprite` dela. Diz se a
/// nuvem já acabou.
fn fade(fart: &mut CFart, sprite: &mut CSprite, dt: f32) -> bool {
    fart.0 -= dt;
    // A vida da nuvem vai de 1, quando nasce, até 0, quando acaba.
    let life = (fart.0 / FART_TIME).max(0.0);
    // Quanto menos vida, maior e mais transparente.
    let size = FART_SIZE * (2.0 - life);
    let scale = Vec2::splat(size) / sprite.texture.size();
    sprite.draw = sprite
        .draw
        .scale(scale)
        .color(Color::new(1.0, 1.0, 1.0, life));
    fart.0 <= 0.0
}
