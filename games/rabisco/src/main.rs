//! Por onde o computador começa quando abrimos o Rabisco.

// É um jogo, não uma biblioteca: não precisa de documentação para outros programas.
#![allow(missing_docs)]

/// Abre a janela do jogo e põe-no a andar até alguém o fechar.
fn main() -> olivia::Result<()> {
    olivia::run::<rabisco::Rabisco>(rabisco::Rabisco::config())
}
