// __forceinline float dequantizeUnsignedPercent(const unsigned int i)
// {
//     enum { N = 100 };
//     return min(precise_divide((float)i, (float)N), 1.f); //!! test: optimize div or does this break precision?
// }

// __forceinline unsigned int quantizeUnsignedPercent(const float x)
// {
//     enum { N = 100, Np1 = 101 };
//     assert(x >= 0.f);
//     return min((unsigned int)(x * (float)Np1), (unsigned int)N);
// }

// We don't have precise_divide, it's using sse
#[inline(always)]
fn precise_divide(a: f32, b: f32) -> f32 {
    (a as f64 / b as f64) as f32
}

#[inline]
pub fn dequantize_unsigned_percent(i: u32) -> f32 {
    const N: f32 = 100.0;
    precise_divide(i as f32, N).min(1.0)
}

#[inline]
pub fn quantize_unsigned_percent(x: f32) -> u32 {
    const N: f32 = 100.0;
    const NP1: f32 = 101.0;
    assert!(x >= 0.0);
    (x * NP1).min(N) as u32
}

// template <unsigned char bits> // bits to map to
// __forceinline float dequantizeUnsigned(const unsigned int i)
// {
//     enum { N = (1 << bits) - 1 };
//     return min(precise_divide((float)i, (float)N), 1.f); //!! test: optimize div or does this break precision?
// }

// template <unsigned char bits> // bits to map to
// __forceinline unsigned int quantizeUnsigned(const float x)
// {
//     enum { N = (1 << bits) - 1, Np1 = (1 << bits) };
//     assert(x >= 0.f);
//     return min((unsigned int)(x * (float)Np1), (unsigned int)N);
// }

#[inline]
pub fn dequantize_unsigned(bits: u8, i: u32) -> f32 {
    let n = (1 << bits) - 1;
    precise_divide(i as f32, n as f32).min(1.0)
}

#[inline]
pub fn quantize_unsigned(bits: u8, x: f32) -> u32 {
    let n = (1 << bits) - 1;
    let np1 = 1 << bits;
    assert!(x >= 0.0);
    (x * np1 as f32).min(n as f32) as u32
}
