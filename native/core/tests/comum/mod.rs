#![allow(dead_code)]

const ALFABETO_BASE64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const fn tabela_base64() -> [u8; 256] {
    let mut tabela = [u8::MAX; 256];
    let mut indice = 0;
    while indice < 64 {
        tabela[ALFABETO_BASE64[indice] as usize] = indice as u8;
        indice += 1;
    }
    tabela
}

const VALORES_BASE64: [u8; 256] = tabela_base64();

pub fn codificar_base64(bytes: &[u8]) -> String {
    let mut saida = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for bloco in bytes.chunks(3) {
        let valor = (bloco[0] as u32) << 16
            | (bloco.get(1).copied().unwrap_or(0) as u32) << 8
            | bloco.get(2).copied().unwrap_or(0) as u32;
        for (posicao, deslocamento) in [18, 12, 6, 0].into_iter().enumerate() {
            if posicao <= bloco.len() {
                saida.push(ALFABETO_BASE64[(valor >> deslocamento & 63) as usize] as char);
            } else {
                saida.push('=');
            }
        }
    }
    saida
}

pub fn decodificar_base64(texto: &str) -> Vec<u8> {
    assert!(
        texto.len() % 4 == 0,
        "base64 com comprimento inválido: {texto:?}"
    );
    let simbolos = texto.trim_end_matches('=');
    assert!(
        texto.len() - simbolos.len() <= 2,
        "preenchimento base64 inválido: {texto:?}"
    );
    let mut bytes = Vec::with_capacity(simbolos.len() * 3 / 4);
    let mut acumulado = 0u32;
    let mut bits = 0;
    for simbolo in simbolos.bytes() {
        let valor = VALORES_BASE64[simbolo as usize];
        assert!(valor != u8::MAX, "símbolo base64 inválido em {texto:?}");
        acumulado = acumulado << 6 | valor as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((acumulado >> bits) as u8);
            acumulado &= (1 << bits) - 1;
        }
    }
    bytes
}

pub struct RandomDotNet {
    tabela: [i32; 56],
    inext: usize,
    inextp: usize,
}

impl RandomDotNet {
    const MBIG: i32 = i32::MAX;
    const MSEED: i32 = 161_803_398;

    pub fn new(semente: i32) -> RandomDotNet {
        let mut tabela = [0i32; 56];
        let subtracao = if semente == i32::MIN {
            i32::MAX
        } else {
            semente.abs()
        };
        let mut mj = Self::MSEED.wrapping_sub(subtracao);
        tabela[55] = mj;
        let mut mk = 1i32;
        for i in 1..55 {
            let ii = 21 * i % 55;
            tabela[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(Self::MBIG);
            }
            mj = tabela[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                tabela[i] = tabela[i].wrapping_sub(tabela[1 + (i + 30) % 55]);
                if tabela[i] < 0 {
                    tabela[i] = tabela[i].wrapping_add(Self::MBIG);
                }
            }
        }
        RandomDotNet {
            tabela,
            inext: 0,
            inextp: 21,
        }
    }

    fn amostra_interna(&mut self) -> i32 {
        let mut inext = self.inext + 1;
        if inext >= 56 {
            inext = 1;
        }
        let mut inextp = self.inextp + 1;
        if inextp >= 56 {
            inextp = 1;
        }
        let mut valor = self.tabela[inext].wrapping_sub(self.tabela[inextp]);
        if valor == Self::MBIG {
            valor -= 1;
        }
        if valor < 0 {
            valor = valor.wrapping_add(Self::MBIG);
        }
        self.tabela[inext] = valor;
        self.inext = inext;
        self.inextp = inextp;
        valor
    }

    pub fn amostra(&mut self) -> f64 {
        self.amostra_interna() as f64 * (1.0 / Self::MBIG as f64)
    }

    fn amostra_para_intervalo_grande(&mut self) -> f64 {
        let mut resultado = self.amostra_interna();
        if self.amostra_interna() % 2 == 0 {
            resultado = -resultado;
        }
        let mut valor = resultado as f64;
        valor += (i32::MAX - 1) as f64;
        valor /= (2 * i32::MAX as u32 - 1) as f64;
        valor
    }

    pub fn next(&mut self, minimo: i32, maximo: i32) -> i32 {
        assert!(minimo <= maximo, "Random.Next: minimo maior que maximo");
        let intervalo = maximo as i64 - minimo as i64;
        if intervalo <= i32::MAX as i64 {
            ((self.amostra() * intervalo as f64) as i32).wrapping_add(minimo)
        } else {
            ((self.amostra_para_intervalo_grande() * intervalo as f64) as i64 + minimo as i64)
                as i32
        }
    }
}
