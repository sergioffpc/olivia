//! O jogo inteiro: liga todas as outras partes e põe-nas a trabalhar juntas.
//!
//! O jogo anda aos bocadinhos, a que chamamos passos. Em cada passo, o jogo vê que teclas estão
//! carregadas, mexe tudo um bocadinho e desenha o resultado. São tantos passos por segundo que
//! parece que tudo se mexe sem parar, como nos desenhos animados.

use olivia::{
    Game, Result, Setup, Update,
    audio::Cue,
    ecs::{CClear, Entity},
    math::{Color, Vec2},
    window::WindowEvent,
};

use crate::{
    assets::{Assets, Musics},
    camera::Camera,
    hud::Hud,
    level::Level,
    random::Random,
    status::{self, CStatus},
};

/// O tamanho da janela do jogo, em píxeis.
pub(crate) const WINDOW: Vec2 = Vec2::new(1280.0, 720.0);
/// A força com que a Terra puxa as coisas para baixo, em metros por segundo a cada segundo.
const GRAVITY: f32 = 9.81;

/// O jogo Rabisco, com tudo o que precisa de lembrar entre um passo e o seguinte.
#[derive(Debug)]
pub struct Rabisco {
    /// A máquina de números à sorte, para escolher os sons.
    random: Random,
    /// Os desenhos, os sons e os movimentos que o jogo carregou.
    assets: Assets,
    /// A música que está a tocar.
    #[expect(dead_code, reason = "ainda nada para nem troca a música")]
    music: Cue,
    /// A câmara que segue o herói.
    camera: Camera,
    /// O nível, com tudo o que lá está.
    level: Level,
    /// As vidas e os pontos desenhados por cima do jogo.
    hud: Hud,
    /// Onde está guardado o placar no mundo do jogo.
    status: Entity,
}

impl Rabisco {
    /// Como é a janela do jogo: o nome, o tamanho e a força da gravidade.
    #[must_use]
    pub fn config() -> olivia::Config {
        olivia::Config::new("rabisco")
            .title("Rabisco")
            .size(WINDOW.x, WINDOW.y)
            .resizable(false)
            // O y cresce para baixo, por isso a gravidade puxa para os y maiores.
            .gravity(0.0, GRAVITY)
    }
}

impl Game for Rabisco {
    /// Prepara o jogo antes do primeiro passo.
    fn load(s: Setup<'_>) -> Result<Self> {
        // Liga a máquina dos números à sorte.
        let mut random = Random::from_clock();
        // Vai buscar os desenhos, os sons e os movimentos, e põe uma música a tocar.
        let assets = Assets::load(s.filesystem, s.graphics, s.audio)?;
        let music = Musics::load(s.filesystem)?.play(s.filesystem, s.audio, &mut random)?;
        // Pinta o céu de azul, e põe no mundo a câmara, o nível, o placar e o que o mostra.
        s.world.spawn((CClear(Color::BLUE),));
        let camera = Camera::spawn(s.world);
        let level = Level::spawn(s.world, &assets)?;
        let status = status::spawn(s.world);
        let hud = Hud::spawn(s.world, &assets.textures, status);

        Ok(Self {
            random,
            assets,
            music,
            camera,
            level,
            hud,
            status,
        })
    }

    /// Um passo do jogo.
    fn update(&mut self, mut u: Update<'_>) -> Result<()> {
        // Se alguém fechou a janela, o jogo acaba.
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        // Vê se já se pode jogar. Se ainda não, espera pelo passo seguinte.
        let Some(mut status) =
            self.level
                .ready_to_play(u.world, u.input, &self.assets, self.status)?
        else {
            return Ok(());
        };
        // Joga um passo no nível, e a câmara vai atrás do herói.
        self.level
            .update(&mut u, &self.assets, &mut self.random, &mut status)?;
        self.camera.update(u.world, &self.level)?;
        // Guarda o placar novo e mostra-o.
        *u.world.get::<&mut CStatus>(self.status)? = status;
        self.hud.update(u.world, &self.assets.textures)?;

        Ok(())
    }
}
