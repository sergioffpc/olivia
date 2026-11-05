//! Números à sorte, para o jogo não ser sempre igual.

/// Uma máquina de números à sorte, como um dado que nunca acaba.
///
/// Os números não são mesmo à sorte: saem de uma conta que os baralha muito bem (chama-se
/// xorshift64*). Mas parecem à sorte, e isso chega para escolher a música ou o som a tocar.
#[derive(Debug)]
pub(crate) struct Random(u64);

impl Random {
    /// Uma máquina que começa na hora do relógio, para cada jogo ser diferente do anterior.
    pub(crate) fn from_clock() -> Self {
        // Usamos o relógio do `web_time`, porque o relógio normal não funciona no navegador.
        let nanos = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        #[allow(clippy::cast_possible_truncation)] // os algarismos do fim são os que mais mudam
        Self::from_seed(nanos as u64)
    }

    /// Uma máquina que começa no número `seed`: com o mesmo número, dá sempre os mesmos números.
    pub(crate) const fn from_seed(seed: u64) -> Self {
        // Começar no zero daria só zeros, por isso garantimos que nunca é zero.
        Self(seed | 1)
    }

    /// O número à sorte seguinte, muito grande.
    pub(crate) fn next_u64(&mut self) -> u64 {
        // Três vezes, empurra os algarismos para um lado e mistura-os com os de antes.
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        // E no fim multiplica por um número especial, para os baralhar ainda melhor.
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Um número à sorte de 0 até `n`, sem chegar a `n`, como um dado com `n` faces.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        // Um dado com zero faces não existe.
        assert!(n > 0, "no number is below 0");
        // Multiplicamos por `n` e ficamos só com a parte de cima: dá um número de 0 até `n`.
        #[allow(clippy::cast_possible_truncation)]
        let pick = ((u128::from(self.next_u64()) * n as u128) >> 64) as usize;
        pick
    }
}
