// ===== AFRI-ED25519 — Test standalone =====
// Notre propre Ed25519 from scratch — souveraineté crypto totale
use sha2::{Sha512, Digest};
use rand::rngs::OsRng;
use rand::RngCore;

// Field element: [u8; 32] little-endian, mod 2^255-19
type Fe = [u8; 32];

// p = 2^255 - 19
const P: [u8; 32] = [
    0xED, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F,
];

fn fe_zero() -> Fe { [0u8; 32] }
fn fe_one() -> Fe { let mut f = [0u8; 32]; f[0] = 1; f }

fn fe_cmp(a: &Fe, b: &Fe) -> i8 {
    for i in (0..32).rev() {
        if a[i] > b[i] { return 1; }
        if a[i] < b[i] { return -1; }
    }
    0
}

fn fe_gte(a: &Fe, b: &Fe) -> bool { fe_cmp(a, b) >= 0 }

fn fe_add_raw(a: &Fe, b: &Fe) -> [u8; 33] {
    let mut r = [0u8; 33];
    let mut carry: u16 = 0;
    for i in 0..32 {
        let s = a[i] as u16 + b[i] as u16 + carry;
        r[i] = s as u8;
        carry = s >> 8;
    }
    r[32] = carry as u8;
    r
}

fn fe_sub_raw(a: &Fe, b: &Fe) -> Fe {
    let mut r = [0u8; 32];
    let mut borrow: i16 = 0;
    for i in 0..32 {
        let s = a[i] as i16 - b[i] as i16 - borrow;
        if s < 0 {
            r[i] = (s + 256) as u8;
            borrow = 1;
        } else {
            r[i] = s as u8;
            borrow = 0;
        }
    }
    r
}

fn fe_add(a: &Fe, b: &Fe) -> Fe {
    let mut r = fe_add_raw(a, b);
    // Handle overflow: 2^256 ≡ 38 mod p
    let carry = r[32] as u64;
    if carry > 0 {
        let to_add = 38 * carry;
        let mut c = to_add;
        for i in 0..32 {
            let s = r[i] as u64 + (c & 0xFF);
            r[i] = s as u8;
            c = (c >> 8) + (s >> 8);
        }
        // c should be 0 since 38*1 = 38 < 256
    }
    // Now reduce: if >= p, subtract p
    let mut result = [0u8; 32];
    result.copy_from_slice(&r[0..32]);
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    result
}

fn fe_sub(a: &Fe, b: &Fe) -> Fe {
    // a - b mod p = a + (2p - b) mod p
    // Use: add a + p, then subtract b
    let a_plus_p = fe_add_raw(a, &P); // a + p, fits in 33 bytes
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(&a_plus_p[0..32]);
    // tmp might have overflow bit in a_plus_p[32], but since a < p and p < 2^255,
    // a + p < 2^256, so a_plus_p[32] is 0 or 1
    // If a_plus_p[32] is 1, we have a + p >= 2^256, which means a >= 2^256 - p = 2^255 + 19
    // But a < p = 2^255 - 19, so a + p < 2^256, so a_plus_p[32] is 0.
    // Actually a < p and p = 2^255 - 19, so a + p < 2*(2^255 - 19) = 2^256 - 38 < 2^256.
    // So a_plus_p[32] is always 0. Good.
    let mut r = fe_sub_raw(&tmp, b);
    // r should be in [0, p), but let's make sure
    if fe_gte(&r, &P) {
        r = fe_sub_raw(&r, &P);
    }
    r
}

fn fe_mul(a: &Fe, b: &Fe) -> Fe {
    // Schoolbook multiplication: 32x8 bits -> [u32; 64]
    let mut acc = [0u32; 64];
    for i in 0..32 {
        for j in 0..32 {
            let prod = (a[i] as u32) * (b[j] as u32);
            acc[i + j] += prod & 0xFFFF;
            acc[i + j + 1] += prod >> 16;
        }
    }
    // Propagate carries
    let mut bytes = [0u8; 64];
    let mut carry: u32 = 0;
    for i in 0..64 {
        let val = acc[i] + carry;
        bytes[i] = (val & 0xFF) as u8;
        carry = val >> 8;
    }

    // Reduce mod 2^255-19
    // Split at byte 32 (bit 256): result = low + 38 * high
    fe_reduce_512(&bytes)
}

fn fe_reduce_512(input: &[u8; 64]) -> Fe {
    // Step 1: result = low[0..32] + 38 * high[32..64]
    let mut r = [0u8; 40]; // enough for 262 bits
    for i in 0..32 {
        r[i] = input[i];
    }
    // Add 38 * high
    let mut carry: u32 = 0;
    for i in 0..32 {
        let val = 38 * (input[32 + i] as u32) + carry + r[i] as u32;
        r[i] = (val & 0xFF) as u8;
        carry = val >> 8;
    }
    let mut idx = 32;
    while carry > 0 && idx < 40 {
        let val = carry + r[idx] as u32;
        r[idx] = (val & 0xFF) as u8;
        carry = val >> 8;
        idx += 1;
    }

    // Step 2: Now r has ~262 bits. Split again at byte 32.
    // result2 = r[0..32] + 38 * r[32..40]
    let mut r2 = [0u8; 33];
    for i in 0..32 {
        r2[i] = r[i];
    }
    let mut carry2: u32 = 0;
    for i in 32..40 {
        let val = 38 * (r[i] as u32) + carry2;
        let pos = i - 32;
        let sum = r2[pos] as u32 + (val & 0xFF);
        r2[pos] = (sum & 0xFF) as u8;
        carry2 = (val >> 8) + (sum >> 8);
    }
    // carry2 might have bits, propagate from pos=8 (next after last modified)
    let mut pos = 8;
    while carry2 > 0 && pos < 33 {
        let sum = r2[pos] as u32 + (carry2 & 0xFF);
        r2[pos] = (sum & 0xFF) as u8;
        carry2 = sum >> 8;
        pos += 1;
    }
    // r2[32] might have overflow — 2^256 ≡ 38 (mod p)
    if r2[32] > 0 {
        let extra = 38 * (r2[32] as u32);
        let mut c = extra;
        for i in 0..32 {
            let sum = r2[i] as u32 + (c & 0xFF);
            r2[i] = (sum & 0xFF) as u8;
            c = (c >> 8) + (sum >> 8);
        }
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&r2[0..32]);

    // Step 3: Conditional subtract p
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    result
}

fn fe_sq(a: &Fe) -> Fe { fe_mul(a, a) }

fn fe_mul_small(a: &Fe, b: u32) -> Fe {
    let mut r = [0u8; 33];
    let mut carry: u32 = 0;
    for i in 0..32 {
        let val = b * (a[i] as u32) + carry;
        r[i] = (val & 0xFF) as u8;
        carry = val >> 8;
    }
    r[32] = carry as u8;
    // Reduce: 2^256 ≡ 38
    let mut result = [0u8; 32];
    if r[32] > 0 {
        let extra = 38 * (r[32] as u32);
        for i in 0..32 {
            result[i] = r[i];
        }
        let mut c = extra;
        for i in 0..32 {
            let s = result[i] as u32 + (c & 0xFF);
            result[i] = (s & 0xFF) as u8;
            c = c >> 8;
        }
    } else {
        result.copy_from_slice(&r[0..32]);
    }
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    if fe_gte(&result, &P) {
        result = fe_sub_raw(&result, &P);
    }
    result
}

fn fe_inv(a: &Fe) -> Fe {
    // a^(p-2) mod p using square-and-multiply
    // p - 2 = 2^255 - 21
    // Use addition chain for p-2
    let mut z = *a;
    // z = a^2
    z = fe_sq(&z);
    // z = a^(2^2)
    z = fe_sq(&z);
    // z = a^(2^2) * a = a^5
    z = fe_mul(&z, a);
    // z = a^(5*2) = a^10
    z = fe_sq(&z);
    // z = a^(10+1) = a^11
    z = fe_mul(&z, a);
    // Now we need a^(2^255 - 21)
    // Let's use the standard addition chain for 2^255 - 21
    // 2^255 - 21 = 2^255 - 2^4 - 2^2 - 1 = 2^255 - 16 - 4 - 1
    // = 2^255 - 21
    // 
    // Standard approach: compute a^(2^255-21) using the chain:
    // Start with z = a^11
    // Then repeatedly square and multiply

    // Actually, let me use a simpler approach: binary exponentiation
    // p - 2 in binary: 2^255 - 21
    // 21 = 16 + 4 + 1 = 10101 in binary
    // So p-2 = 2^255 - 21 = (2^255 - 1) - 20 = all 1s except bits 2 and 4
    
    // Actually let me just use the straightforward binary method
    // p - 2 = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEB
    // In little-endian bytes: [0xEB, 0xFF, 0xFF, ..., 0x7F]
    
    let exp: [u8; 32] = [
        0xEB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F,
    ];
    
    // Square-and-multiply
    let mut result = fe_one();
    for i in (0..32).rev() {
        for j in (0..8).rev() {
            result = fe_sq(&result);
            if (exp[i] >> j) & 1 == 1 {
                result = fe_mul(&result, a);
            }
        }
    }
    result
}

fn fe_neg(a: &Fe) -> Fe {
    fe_sub(&fe_zero(), a)
}

fn fe_from_bytes(b: &[u8; 32]) -> Fe {
    let mut f = [0u8; 32];
    f.copy_from_slice(b);
    // Reduce if needed
    if fe_gte(&f, &P) {
        f = fe_sub_raw(&f, &P);
    }
    f
}

fn fe_to_bytes(a: &Fe) -> [u8; 32] { *a }

// ===== Scalar arithmetic mod L =====
// L = 2^252 + 27742317777372353535851937790883648493
const L: [u8; 32] = [
    0xED, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
    0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

fn sc_gte(a: &[u8; 32], b: &[u8; 32]) -> bool {
    for i in (0..32).rev() {
        if a[i] > b[i] { return true; }
        if a[i] < b[i] { return false; }
    }
    true
}

fn sc_sub(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut r = [0u8; 32];
    let mut borrow: i16 = 0;
    for i in 0..32 {
        let s = a[i] as i16 - b[i] as i16 - borrow;
        if s < 0 {
            r[i] = (s + 256) as u8;
            borrow = 1;
        } else {
            r[i] = s as u8;
            borrow = 0;
        }
    }
    r
}

fn sc_add(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut r = [0u8; 33];
    let mut carry: u16 = 0;
    for i in 0..32 {
        let s = a[i] as u16 + b[i] as u16 + carry;
        r[i] = s as u8;
        carry = s >> 8;
    }
    r[32] = carry as u8;
    // Reduce mod L if needed
    let mut result = [0u8; 32];
    result.copy_from_slice(&r[0..32]);
    if r[32] > 0 || sc_gte(&result, &L) {
        result = sc_sub(&result, &L);
    }
    if sc_gte(&result, &L) {
        result = sc_sub(&result, &L);
    }
    result
}

fn sc_mul(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    // Schoolbook multiplication into [u32; 64]
    let mut acc = [0u32; 64];
    for i in 0..32 {
        for j in 0..32 {
            let prod = (a[i] as u32) * (b[j] as u32);
            acc[i + j] += prod & 0xFFFF;
            acc[i + j + 1] += prod >> 16;
        }
    }
    // Propagate carries
    let mut bytes = [0u8; 64];
    let mut carry: u32 = 0;
    for i in 0..64 {
        let val = acc[i] + carry;
        bytes[i] = (val & 0xFF) as u8;
        carry = val >> 8;
    }
    
    // Reduce mod L
    sc_reduce_512(&bytes)
}

fn sc_reduce_512(input: &[u8; 64]) -> [u8; 32] {
    // L = 2^252 + c where c = 27742317777372353535851937790883648493
    // 2^252 ≡ -c mod L
    // 2^256 = 2^4 * 2^252 ≡ -16c mod L
    
    // c in bytes (little-endian):
    let c: [u8; 32] = [
        0xED, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
        0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ];
    // Note: c = L (since L = 2^252 + c, and L < 2^253, so c = L - 2^252)
    // Actually L = 2^252 + 27742317777372353535851937790883648493
    // So c = L - 2^252 = 27742317777372353535851937790883648493
    // And 2^252 ≡ -c ≡ -L + 2^252 mod L... that's circular.
    // Let me think differently.
    
    // 2^256 mod L:
    // 2^256 = 2^4 * 2^252 = 16 * 2^252
    // 2^252 = L - c where c = 27742317777372353535851937790883648493
    // So 2^256 = 16 * (L - c) = 16L - 16c
    // 2^256 mod L = -16c mod L = L - 16c (if 16c < L)
    // 16c = 16 * 27742317777372353535851937790883648493
    //      = 443877084437957565573631004654138375888
    // L = 72370055773322622139731865630429942408571163593799096060013099666552671763881
    // 16c ≈ 4.4 * 10^38, L ≈ 7.2 * 10^75, so 16c << L. Good.
    
    // So 2^256 mod L = L - 16c
    // Let me compute 16c:
    // c = 27742317777372353535851937790883648493
    // 16c = 443877084437957565573631004654138375888
    
    // In bytes (little-endian), 16c:
    // Let me compute: 443877084437957565573631004654138375888
    // = 0x1543D0E9A4F8B41B3B7B7B7B7B7B7B7B7B7B7B7B0
    // Hmm, I can't easily compute this by hand. Let me use a different approach.
    
    // Alternative: just do repeated subtraction.
    // Since the input is 512 bits and L is 253 bits, I need to reduce by ~259 bits.
    // I can do this by processing 8 bits at a time from the top.
    
    // Actually, the simplest correct approach: 
    // 1. Start with the 512-bit number
    // 2. For each bit from bit 511 down to bit 253:
    //    - If that bit is set, subtract L shifted left by (bit_position - 252)
    //    This is O(259) subtractions of 256-bit numbers. Slow but correct.
    
    // Even simpler: just do mod by repeated subtraction of L * 2^n
    // But that's O(2^259) which is way too slow.
    
    // OK, let me use the proper approach:
    // n = low + high * 2^256
    // 2^256 mod L = L - 16c where c = L - 2^252
    // So n mod L = (low + high * (L - 16c)) mod L = (low - 16c * high) mod L
    
    // 16c = 16 * (L - 2^252) = 16L - 2^256
    // So 2^256 mod L = L - (2^256 - 16L) mod L... 
    // Actually: 2^256 = 16 * 2^252 = 16 * (L - c) = 16L - 16c
    // 2^256 mod L = -16c mod L = L - 16c
    
    // So I need to compute L - 16c = L - 16*(L - 2^252) = L - 16L + 16*2^252 = -15L + 2^256
    // That's just 2^256 mod L again... circular.
    
    // Let me just compute 16c directly.
    // c = 27742317777372353535851937790883648493
    // 16c = 443877084437957565573631004654138375888
    
    // In hex: let me compute
    // c = 0x0000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED
    // (this is L in little-endian, with the top byte 0x10 becoming 0x00 since we subtract 2^252)
    // Wait, L = 2^252 + c, so c = L - 2^252
    // L in big-endian hex: 0x1000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED
    // 2^252 in big-endian: 0x1000000000000000000000000000000000000000000000000000000000000000
    // c = L - 2^252 = 0x0000000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED
    // 16c = 0x00000000000000000000000000000014DEF9DEA2F79CD65812631A5CF5D3ED0
    // (shifted left by 4 bits)
    
    // In little-endian bytes, 16c:
    // 0xED, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
    // 0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
    // 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // Wait, that's just c with a 0 appended (shifted left 4 bits = 1 nibble)
    // Actually, 16c = c << 4, which in bytes means shifting left by 4 bits = half a byte
    
    // This is getting too complicated. Let me use a completely different approach.
    
    // SIMPLE APPROACH: Use the Barrett reduction or just do it byte by byte.
    // 
    // Actually, the simplest correct approach for scalar reduction:
    // Just compute low + high * (2^256 mod L) and then reduce the result.
    // 
    // 2^256 mod L: I'll compute this at runtime.
    
    // Let me precompute 2^256 mod L by computing L - 16*(L - 2^252)
    // = L - 16*L + 16*2^252 = -15*L + 2^256
    // Hmm, that's circular again.
    
    // OK, let me just compute 2^256 mod L directly.
    // 2^256 = 2^4 * 2^252 = 16 * 2^252
    // 2^252 mod L = 2^252 (since 2^252 < L)
    // So 2^256 mod L = 16 * 2^252 mod L = 16 * (L - c) mod L = -16c mod L = L - 16c
    
    // c = L - 2^252
    // In little-endian bytes, c is L with byte[31] changed from 0x10 to 0x00:
    let c_bytes: [u8; 32] = [
        0xED, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
        0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    
    // 16c = c << 4 (shift left by 4 bits)
    let mut c16 = [0u8; 33];
    let mut carry: u8 = 0;
    for i in 0..32 {
        let val = (c_bytes[i] as u16) << 4 | (carry as u16);
        c16[i] = (val & 0xFF) as u8;
        carry = (val >> 8) as u8;
    }
    c16[32] = carry;
    
    // 2^256 mod L = L - 16c (since 16c < L)
    // But 16c might be 33 bytes. Let me check: c < 2^125 (since c ≈ 2.77 * 10^38 ≈ 2^125)
    // So 16c < 2^129, which is 33 bytes. But L ≈ 2^253, so 16c << L.
    // L - 16c: just subtract c16 from L
    
    let mut r256 = [0u8; 32]; // 2^256 mod L
    let mut borrow: i16 = 0;
    for i in 0..32 {
        let s = L[i] as i16 - c16[i] as i16 - borrow;
        if s < 0 {
            r256[i] = (s + 256) as u8;
            borrow = 1;
        } else {
            r256[i] = s as u8;
            borrow = 0;
        }
    }
    // borrow should be 0 since 16c < L
    
    // Now: n mod L = (low + high * r256) mod L
    // low = input[0..32], high = input[32..64]
    // high * r256 is a 512-bit product, but high < 2^256 and r256 < L < 2^253
    // So high * r256 < 2^509
    // low + high * r256 < 2^512
    
    // Hmm, this doesn't help much. We still have a 512-bit number.
    // We need to iterate the reduction.
    
    // Actually, high < 2^256 and r256 < 2^253, so high * r256 < 2^509
    // low < 2^256, so low + high*r256 < 2^509 + 2^256 ≈ 2^509
    // This is still 509 bits, not much better than 512.
    
    // The issue is that one round of reduction only removes ~3 bits (512 -> 509).
    // We need ~85 rounds to get from 512 to 253 bits. That's too many.
    
    // Better approach: process the high part in chunks.
    // Split the 512-bit input into 128-bit chunks:
    // n = n0 + n1 * 2^128 + n2 * 2^256 + n3 * 2^384
    // 2^256 mod L = r256 (precomputed above)
    // 2^384 = 2^128 * 2^256, so 2^384 mod L = (2^128 * r256) mod L
    // This still requires multi-precision multiplication.
    
    // OK, let me use a completely different approach. 
    // I'll use the standard Ed25519 scalar reduction from the reference implementation.
    
    // The reference implementation reduces a 512-bit number mod L using the fact that
    // L = 2^252 + 27742317777372353535851937790883648493
    // and processes the number in 5-bit chunks from the top.
    
    // Actually, the simplest approach that works: just do it in two rounds.
    // Round 1: n = low(256) + high(256) * r256(253) → result is ~509 bits
    // Round 2: split the 509-bit result into low2(256) + high2(253) * r256 → result is ~506 bits
    // ... this converges too slowly.
    
    // Let me try a different chunking. Split into 64-bit chunks:
    // n = sum of limb[i] * 2^(64*i) for i in 0..8
    // 2^64 mod L, 2^128 mod L, 2^192 mod L, 2^256 mod L, etc.
    // Precompute all powers of 2^64 mod L, then:
    // n mod L = sum of limb[i] * (2^(64*i) mod L) mod L
    // Each term is 64-bit * 253-bit = 317-bit, and we have 8 terms
    // Sum is ~320 bits. Then reduce the 320-bit result.
    // Split into 64-bit chunks again, repeat.
    // After 2 rounds: 320 -> ~70 bits. One more subtraction. Done.
    
    // This is the approach I'll use. Let me precompute 2^(64*k) mod L for k=0..7.
    
    // Actually, I realize I'm overcomplicating this. Let me just use the approach of
    // processing bit by bit from the top, which is O(512) operations but each is simple.
    
    // Bit-by-bit modular reduction:
    // Start with r = 0
    // For each bit from MSB to LSB:
    //   r = r * 2 + bit
    //   if r >= L: r -= L
    // This gives n mod L. Simple and correct.
    // O(512) iterations, each with a 256-bit shift+add and compare+subtract.
    // Each iteration is O(32) byte operations. Total: O(512 * 32) = O(16384) operations.
    // This is fine for our use case.
    
    let mut r = [0u8; 32];
    for i in (0..512).rev() {
        // r = r * 2
        let mut carry: u8 = 0;
        for j in 0..32 {
            let val = (r[j] as u16) << 1 | (carry as u16);
            r[j] = (val & 0xFF) as u8;
            carry = (val >> 8) as u8;
        }
        // Add the current bit
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        let bit = (input[byte_idx] >> bit_idx) & 1;
        if bit == 1 {
            // r = r + 1
            let mut c: u16 = 1;
            for j in 0..32 {
                let s = r[j] as u16 + c;
                r[j] = (s & 0xFF) as u8;
                c = s >> 8;
            }
        }
        // If r >= L, subtract L
        if sc_gte(&r, &L) {
            r = sc_sub(&r, &L);
        }
    }
    r
}

// ===== Point operations on Edwards curve =====
// Curve: -x^2 + y^2 = 1 + d*x^2*y^2
// Using extended coordinates: (X, Y, Z, T) where x=X/Z, y=Y/Z, T=XY/Z

#[derive(Clone, Copy)]
struct Point { x: Fe, y: Fe, z: Fe, t: Fe }

// d = -121665/121666 mod p
// Precomputed: d = 37095705934669439343138083508754565189542113879843219016388785533085940283555
// In little-endian bytes:
const D_BYTES: [u8; 32] = [
    0xA3, 0x78, 0x59, 0x35, 0x9C, 0xA6, 0xEA, 0xCD,
    0x4D, 0x4B, 0xE8, 0xB5, 0x9A, 0x5F, 0x6E, 0x7E,
    0x0C, 0x3F, 0x0C, 0x9E, 0x81, 0x57, 0x8E, 0x3C,
    0x9C, 0x4D, 0x4D, 0x4D, 0x4D, 0x4D, 0x4D, 0x2C,
];

// Actually, I'm not confident in these d bytes. Let me compute d at runtime.
// d = -121665 * inv(121666) mod p = (p - 121665) * inv(121666) mod p

fn compute_d() -> Fe {
    let mut numerator = fe_zero();
    // p - 121665
    numerator[0] = 0xED; // p[0] = 0xED
    // Actually, p = 2^255 - 19, so p - 121665 = 2^255 - 19 - 121665 = 2^255 - 121684
    // Let me compute this properly
    // p in bytes: [0xED, 0xFF, ..., 0x7F]
    // 121665 = 0x1DBA1
    // p - 121665:
    let p_minus: Fe = {
        let mut r = P;
        let mut borrow: i16 = 0;
        let val = 121665u32;
        let b0 = (val & 0xFF) as u8;       // 0xA1
        let b1 = ((val >> 8) & 0xFF) as u8; // 0xDB
        let b2 = ((val >> 16) & 0xFF) as u8; // 0x01
        let sub_bytes = [b0, b1, b2];
        for i in 0..3 {
            let s = r[i] as i16 - sub_bytes[i] as i16 - borrow;
            if s < 0 {
                r[i] = (s + 256) as u8;
                borrow = 1;
            } else {
                r[i] = s as u8;
                borrow = 0;
            }
        }
        // Continue borrow if needed
        let mut i = 3;
        while borrow > 0 && i < 32 {
            let s = r[i] as i16 - borrow;
            if s < 0 {
                r[i] = (s + 256) as u8;
                borrow = 1;
            } else {
                r[i] = s as u8;
                borrow = 0;
            }
            i += 1;
        }
        r
    };
    
    // denominator = 121666 = 0x1DB42
    // Little-endian: 0x42, 0xDB, 0x01
    let mut denom = fe_zero();
    denom[0] = 0x42;
    denom[1] = 0xDB;
    denom[2] = 0x01;
    
    let denom_inv = fe_inv(&denom);
    fe_mul(&p_minus, &denom_inv)
}

// 2*d
fn compute_2d(d: &Fe) -> Fe {
    fe_add(d, d)
}

fn point_identity() -> Point {
    Point { x: fe_zero(), y: fe_one(), z: fe_one(), t: fe_zero() }
}

fn point_add(p: &Point, q: &Point, d: &Fe, d2: &Fe) -> Point {
    // Extended twisted Edwards addition (a=-1)
    // From https://en.wikipedia.org/wiki/Twisted_Edwards_curves#Addition_on_twisted_Edwards_curves
    // For a=-1:
    // A = (Y1-X1)*(Y2-X2)
    // B = (Y1+X1)*(Y2+X2)
    // C = T1*2d*T2
    // D = Z1*2*Z2
    // E = B-A
    // F = D-C
    // G = D+C
    // H = B+A
    // X3 = E*F
    // Y3 = G*H
    // T3 = E*H
    // Z3 = F*G
    
    let y1_minus_x1 = fe_sub(&p.y, &p.x);
    let y2_minus_x2 = fe_sub(&q.y, &q.x);
    let a = fe_mul(&y1_minus_x1, &y2_minus_x2);
    
    let y1_plus_x1 = fe_add(&p.y, &p.x);
    let y2_plus_x2 = fe_add(&q.y, &q.x);
    let b = fe_mul(&y1_plus_x1, &y2_plus_x2);
    
    let t1_2d = fe_mul(&p.t, d2);
    let c = fe_mul(&t1_2d, &q.t);
    
    let d_val = fe_add(&p.z, &q.z); // Z1*2*Z2 = 2*Z1*Z2, but we use Z1+Z2 then multiply by... 
    // Actually, D = Z1 * 2 * Z2 = 2 * Z1 * Z2
    // Let me compute it as: D = fe_mul(&fe_add(&p.z, &p.z), &q.z) or fe_mul(&p.z, &fe_add(&q.z, &q.z))
    // Or just: D = fe_mul(&p.z, &q.z) then double it
    let d_val = fe_mul(&p.z, &q.z);
    let d_val = fe_add(&d_val, &d_val); // 2*Z1*Z2
    
    let e = fe_sub(&b, &a);
    let f = fe_sub(&d_val, &c);
    let g = fe_add(&d_val, &c);
    let h = fe_add(&b, &a);
    
    let x3 = fe_mul(&e, &f);
    let y3 = fe_mul(&g, &h);
    let t3 = fe_mul(&e, &h);
    let z3 = fe_mul(&f, &g);
    
    Point { x: x3, y: y3, z: z3, t: t3 }
}

fn point_double(p: &Point, d: &Fe, d2: &Fe) -> Point {
    // For Edwards curves, doubling is the same as addition with itself
    point_add(p, p, d, d2)
}

fn point_scalar_mul(scalar: &[u8; 32], point: &Point, d: &Fe, d2: &Fe) -> Point {
    // Double-and-add
    let mut result = point_identity();
    // Process from MSB to LSB
    for i in (0..32).rev() {
        for j in (0..8).rev() {
            result = point_double(&result, d, d2);
            if (scalar[i] >> j) & 1 == 1 {
                result = point_add(&result, point, d, d2);
            }
        }
    }
    result
}

fn point_compress(p: &Point) -> [u8; 32] {
    // Compute x = X/Z, y = Y/Z
    let z_inv = fe_inv(&p.z);
    let x = fe_mul(&p.x, &z_inv);
    let y = fe_mul(&p.y, &z_inv);
    
    let mut bytes = fe_to_bytes(&y);
    // Set the high bit of the last byte to the sign of x (bit 0 of x)
    if x[0] & 1 == 1 {
        bytes[31] |= 0x80;
    }
    bytes
}

fn point_decompress(bytes: &[u8; 32], d: &Fe) -> Option<Point> {
    // Recover x from y
    let sign_bit = (bytes[31] >> 7) & 1;
    let mut y_bytes = *bytes;
    y_bytes[31] &= 0x7F; // Clear sign bit
    let y = fe_from_bytes(&y_bytes);
    
    // x^2 = (y^2 - 1) / (d*y^2 + 1) mod p
    // For a=-1: -x^2 + y^2 = 1 + d*x^2*y^2
    // => y^2 - 1 = x^2 * (d*y^2 + 1)
    // => x^2 = (y^2 - 1) / (d*y^2 + 1)
    
    let y2 = fe_sq(&y);
    let u = fe_sub(&y2, &fe_one()); // y^2 - 1
    let v = fe_add(&fe_mul(d, &y2), &fe_one()); // d*y^2 + 1
    
    // x^2 = u * v^(-1) = u * v^(p-2) mod p
    let v_inv = fe_inv(&v);
    let x2 = fe_mul(&u, &v_inv);
    
    // Compute x = sqrt(x2)
    // For p ≡ 5 mod 8 (which 2^255-19 is):
    // sqrt(a) = a^((p+3)/8) if a is a QR
    // Check: if x2^((p-1)/2) = 1, then a is a QR
    
    // (p+3)/8 = (2^255 - 19 + 3) / 8 = (2^255 - 16) / 8 = 2^252 - 2
    let mut exp = [0u8; 32];
    exp[0] = 0xFE; // 2^252 - 2 in little-endian
    // 2^252 - 2 = 0x1000000000000000000000000000000000000000000000000000000000000000 - 2
    // = 0x0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE
    // Little-endian: [0xFE, 0xFF, 0xFF, ..., 0x0F]
    exp[0] = 0xFE;
    for i in 1..31 {
        exp[i] = 0xFF;
    }
    exp[31] = 0x0F;
    
    let mut x = fe_sq(&x2); // x2^2
    // Now compute x2^((p+3)/8) using square-and-multiply
    let mut result = fe_one();
    for i in (0..32).rev() {
        for j in (0..8).rev() {
            result = fe_sq(&result);
            if (exp[i] >> j) & 1 == 1 {
                result = fe_mul(&result, &x2);
            }
        }
    }
    x = result;
    
    // Check: x^2 == x2?
    let x_sq = fe_sq(&x);
    if fe_cmp(&x_sq, &x2) != 0 {
        // Try x * sqrt(-1)
        // sqrt(-1) = 2^((p-1)/4) mod p
        // (p-1)/4 = (2^255 - 20) / 4 = 2^253 - 5
        let mut exp2 = [0u8; 32];
        exp2[0] = 0xFB; // 2^253 - 5 in little-endian
        // 2^253 - 5 = 0x2000000000000000000000000000000000000000000000000000000000000000 - 5
        // = 0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFB
        // Little-endian: [0xFB, 0xFF, ..., 0x1F]
        exp2[0] = 0xFB;
        for i in 1..31 {
            exp2[i] = 0xFF;
        }
        exp2[31] = 0x1F;
        
        let mut sqrt_m1 = fe_one();
        let mut base = fe_zero();
        base[0] = 2; // 2, not -1! sqrt(-1) = 2^((p-1)/4) mod p
        for i in (0..32).rev() {
            for j in (0..8).rev() {
                sqrt_m1 = fe_sq(&sqrt_m1);
                if (exp2[i] >> j) & 1 == 1 {
                    sqrt_m1 = fe_mul(&sqrt_m1, &base);
                }
            }
        }
        
        x = fe_mul(&x, &sqrt_m1);
        let x_sq2 = fe_sq(&x);
        if fe_cmp(&x_sq2, &x2) != 0 {
            return None; // Not a valid point
        }
    }
    
    // Adjust sign
    if (x[0] & 1) != sign_bit {
        x = fe_neg(&x);
    }
    
    let t = fe_mul(&x, &y);
    Some(Point { x, y, z: fe_one(), t })
}

// ===== Ed25519 Key Types =====

pub struct AfriSecretKey { seed: [u8; 32] }
pub struct AfriPublicKey { bytes: [u8; 32] }
pub struct AfriSignature { bytes: [u8; 64] }

impl AfriSecretKey {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        AfriSecretKey { seed }
    }
    
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        AfriSecretKey { seed: *bytes }
    }
    
    pub fn to_bytes(&self) -> [u8; 32] {
        self.seed
    }
    
    pub fn verifying_key(&self) -> AfriPublicKey {
        let d = compute_d();
        let d2 = compute_2d(&d);
        
        // Hash seed with SHA-512
        let h = Sha512::digest(&self.seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&h[0..32]);
        
        // Clamp: clear bits 0, 1, 2 of byte 0, set bit 2 of byte 31, clear bits 3-7 of byte 31
        scalar[0] &= 0xF8;
        scalar[31] &= 0x7F;
        scalar[31] |= 0x40;
        
        // Compute A = a * B
        let b = compute_base_point(&d, &d2);
        let a_point = point_scalar_mul(&scalar, &b, &d, &d2);
        let pub_bytes = point_compress(&a_point);
        
        AfriPublicKey { bytes: pub_bytes }
    }
    
    pub fn sign(&self, message: &[u8]) -> AfriSignature {
        let d = compute_d();
        let d2 = compute_2d(&d);
        let b = compute_base_point(&d, &d2);
        
        // Hash seed with SHA-512
        let h = Sha512::digest(&self.seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&h[0..32]);
        let prefix = &h[32..64];
        
        // Clamp
        scalar[0] &= 0xF8;
        scalar[31] &= 0x7F;
        scalar[31] |= 0x40;
        
        // Public key A
        let a_point = point_scalar_mul(&scalar, &b, &d, &d2);
        let a_bytes = point_compress(&a_point);
        
        // r = SHA-512(prefix || M) mod L
        let mut hasher = Sha512::new();
        hasher.update(prefix);
        hasher.update(message);
        let r_hash = hasher.finalize();
        let mut r = [0u8; 32];
        r.copy_from_slice(&r_hash[0..32]);
        r = sc_reduce_512(&{
            let mut full = [0u8; 64];
            full.copy_from_slice(&r_hash);
            full
        });
        
        // R = r * B
        let r_point = point_scalar_mul(&r, &b, &d, &d2);
        let r_bytes = point_compress(&r_point);
        
        // k = SHA-512(R || A || M) mod L
        let mut hasher2 = Sha512::new();
        hasher2.update(&r_bytes);
        hasher2.update(&a_bytes);
        hasher2.update(message);
        let k_hash = hasher2.finalize();
        let mut k = [0u8; 32];
        k.copy_from_slice(&k_hash[0..32]);
        k = sc_reduce_512(&{
            let mut full = [0u8; 64];
            full.copy_from_slice(&k_hash);
            full
        });
        
        // S = (r + k * a) mod L
        let ka = sc_mul(&k, &scalar);
        let s = sc_add(&r, &ka);
        
        // Signature = R || S
        let mut sig = [0u8; 64];
        sig[0..32].copy_from_slice(&r_bytes);
        sig[32..64].copy_from_slice(&s);
        
        AfriSignature { bytes: sig }
    }
}

impl AfriPublicKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let d = compute_d();
        // Verify it's a valid point
        match point_decompress(bytes, &d) {
            Some(_) => Some(AfriPublicKey { bytes: *bytes }),
            None => None,
        }
    }
    
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }
    
    pub fn verify(&self, message: &[u8], sig: &AfriSignature) -> bool {
        let d = compute_d();
        let d2 = compute_2d(&d);
        let b = compute_base_point(&d, &d2);
        
        let r_bytes: [u8; 32] = sig.bytes[0..32].try_into().unwrap();
        let s_bytes: [u8; 32] = sig.bytes[32..64].try_into().unwrap();
        
        // Check S < L
        if sc_gte(&s_bytes, &L) {
            return false;
        }
        
        // Decompress A (public key)
        let a_point = match point_decompress(&self.bytes, &d) {
            Some(p) => p,
            None => return false,
        };
        
        // Decompress R
        let r_point = match point_decompress(&r_bytes, &d) {
            Some(p) => p,
            None => return false,
        };
        
        // k = SHA-512(R || A || M) mod L
        let mut hasher = Sha512::new();
        hasher.update(&r_bytes);
        hasher.update(&self.bytes);
        hasher.update(message);
        let k_hash = hasher.finalize();
        let mut k = [0u8; 32];
        k.copy_from_slice(&k_hash[0..32]);
        k = sc_reduce_512(&{
            let mut full = [0u8; 64];
            full.copy_from_slice(&k_hash);
            full
        });
        
        // Check: S * B == R + k * A
        let sb = point_scalar_mul(&s_bytes, &b, &d, &d2);
        let ka = point_scalar_mul(&k, &a_point, &d, &d2);
        let r_plus_ka = point_add(&r_point, &ka, &d, &d2);
        
        // Compare compressed forms
        let sb_bytes = point_compress(&sb);
        let rka_bytes = point_compress(&r_plus_ka);
        
        sb_bytes == rka_bytes
    }
}

impl AfriSignature {
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        AfriSignature { bytes: *bytes }
    }
    
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }
}

fn compute_base_point(d: &Fe, d2: &Fe) -> Point {
    // B.y = 4/5 mod p
    let mut four = fe_zero();
    four[0] = 4;
    let mut five = fe_zero();
    five[0] = 5;
    let five_inv = fe_inv(&five);
    let by = fe_mul(&four, &five_inv);
    
    // B.x = sqrt((y^2 - 1) / (d*y^2 + 1))
    let y2 = fe_sq(&by);
    let u = fe_sub(&y2, &fe_one());
    let v = fe_add(&fe_mul(d, &y2), &fe_one());
    let v_inv = fe_inv(&v);
    let x2 = fe_mul(&u, &v_inv);
    
    // sqrt(x2) = x2^((p+3)/8)
    let mut exp = [0u8; 32];
    exp[0] = 0xFE;
    for i in 1..31 { exp[i] = 0xFF; }
    exp[31] = 0x0F;
    
    let mut x = fe_one();
    for i in (0..32).rev() {
        for j in (0..8).rev() {
            x = fe_sq(&x);
            if (exp[i] >> j) & 1 == 1 {
                x = fe_mul(&x, &x2);
            }
        }
    }
    
    // Check x^2 == x2
    let x_sq = fe_sq(&x);
    if fe_cmp(&x_sq, &x2) != 0 {
        // x = x * sqrt(-1)
        let mut exp2 = [0u8; 32];
        exp2[0] = 0xFB;
        for i in 1..31 { exp2[i] = 0xFF; }
        exp2[31] = 0x1F;
        
        let mut sqrt_m1 = fe_one();
        let mut base = fe_zero();
        base[0] = 2; // 2, not -1! sqrt(-1) = 2^((p-1)/4) mod p
        for i in (0..32).rev() {
            for j in (0..8).rev() {
                sqrt_m1 = fe_sq(&sqrt_m1);
                if (exp2[i] >> j) & 1 == 1 {
                    sqrt_m1 = fe_mul(&sqrt_m1, &base);
                }
            }
        }
        x = fe_mul(&x, &sqrt_m1);
    }
    
    // Choose positive root (even x)
    if x[0] & 1 == 1 {
        x = fe_neg(&x);
    }
    
    let t = fe_mul(&x, &by);
    Point { x, y: by, z: fe_one(), t }
}

// ===== TEST =====
fn main() {
    println!("🔧 Afri-Ed25519 — Debug de notre crypto from scratch");
    println!("");
    
    round_trip_test();
}
// ===== ROUND-TRIP TEST =====
fn round_trip_test() {
    println!("🔧 Afri-Ed25519 — Test complet de notre crypto from scratch");
    println!("");
    
    let d = compute_d();
    let d2 = compute_2d(&d);
    let b = compute_base_point(&d, &d2);
    
    // Test 0: point_add with different points
    println!("📋 Test 0: point_add(3B, 5B) == 8B...");
    let p3 = point_scalar_mul(&[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &b, &d, &d2);
    let p5 = point_scalar_mul(&[5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &b, &d, &d2);
    let p8 = point_scalar_mul(&[8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &b, &d, &d2);
    let p3_plus_p5 = point_add(&p3, &p5, &d, &d2);
    let p3p5_c = point_compress(&p3_plus_p5);
    let p8_c = point_compress(&p8);
    println!("  3B+5B: {}", hex::encode(&p3p5_c[..8]));
    println!("  8B:    {}", hex::encode(&p8_c[..8]));
    println!("  Match: {}", if p3p5_c == p8_c { "✅" } else { "❌" });
    
    // Test 0b: point_add with decompressed points
    println!("");
    println!("📋 Test 0b: point_add(decomp(3B), decomp(5B)) == 8B...");
    let p3_c = point_compress(&p3);
    let p5_c = point_compress(&p5);
    let p3_dec = point_decompress(&p3_c, &d).unwrap();
    let p5_dec = point_decompress(&p5_c, &d).unwrap();
    let p3d_plus_p5d = point_add(&p3_dec, &p5_dec, &d, &d2);
    let p3d5d_c = point_compress(&p3d_plus_p5d);
    println!("  decomp(3B)+decomp(5B): {}", hex::encode(&p3d5d_c[..8]));
    println!("  8B:                   {}", hex::encode(&p8_c[..8]));
    println!("  Match: {}", if p3d5d_c == p8_c { "✅" } else { "❌" });
    
    // Test 0c: point_add(B, B) == 2B
    println!("");
    println!("📋 Test 0c: point_add(B, B) == 2B...");
    let p2 = point_scalar_mul(&[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &b, &d, &d2);
    let b_plus_b = point_add(&b, &b, &d, &d2);
    let bb_c = point_compress(&b_plus_b);
    let p2_c = point_compress(&p2);
    println!("  B+B: {}", hex::encode(&bb_c[..8]));
    println!("  2B:  {}", hex::encode(&p2_c[..8]));
    println!("  Match: {}", if bb_c == p2_c { "✅" } else { "❌" });

    // Test 0d: point_scalar_mul with non-B base point
    // 3 * (5B) should equal 15B
    println!("");
    println!("📋 Test 0d: point_scalar_mul(3, 5B) == 15B...");
    let p15 = point_scalar_mul(&[15,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &b, &d, &d2);
    let p3_times_5b = point_scalar_mul(&[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &p5, &d, &d2);
    let p15_c = point_compress(&p15);
    let p3x5_c = point_compress(&p3_times_5b);
    println!("  3*(5B): {}", hex::encode(&p3x5_c[..8]));
    println!("  15B:    {}", hex::encode(&p15_c[..8]));
    println!("  Match:  {}", if p3x5_c == p15_c { "✅" } else { "❌" });

    // Test 0e: point_scalar_mul with decompressed non-B base
    // 3 * decomp(5B) should equal 15B
    println!("");
    println!("📋 Test 0e: point_scalar_mul(3, decomp(5B)) == 15B...");
    let p3x5d = point_scalar_mul(&[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], &p5_dec, &d, &d2);
    let p3x5d_c = point_compress(&p3x5d);
    println!("  3*decomp(5B): {}", hex::encode(&p3x5d_c[..8]));
    println!("  15B:          {}", hex::encode(&p15_c[..8]));
    println!("  Match:         {}", if p3x5d_c == p15_c { "✅" } else { "❌" });
    
    // Test 0f: fe_mul correctness tests
    println!("");
    println!("📋 Test 0f: fe_mul correctness...");
    {
        // (p-1)*(p-1) mod p = (-1)^2 = 1
        let p_minus_1 = {
            let mut v = P;
            v[0] -= 1; // p-1
            v
        };
        let r1 = fe_mul(&p_minus_1, &p_minus_1);
        println!("  (p-1)^2: {} (expect 01000000...)", hex::encode(&r1[..4]));
        println!("  (p-1)^2==1: {}", if r1 == fe_one() { "✅" } else { "❌" });

        // (p-1)*2 mod p = -2 mod p = p-2
        let two = { let mut v = fe_zero(); v[0] = 2; v };
        let r2 = fe_mul(&p_minus_1, &two);
        let p_minus_2 = { let mut v = P; v[0] -= 2; v };
        println!("  (p-1)*2: {} (expect {})", hex::encode(&r2[..4]), hex::encode(&p_minus_2[..4]));
        println!("  (p-1)*2==p-2: {}", if r2 == p_minus_2 { "✅" } else { "❌" });

        // 3*5 = 15
        let three = { let mut v = fe_zero(); v[0] = 3; v };
        let five = { let mut v = fe_zero(); v[0] = 5; v };
        let r3 = fe_mul(&three, &five);
        let fifteen = { let mut v = fe_zero(); v[0] = 15; v };
        println!("  3*5: {} (expect {})", hex::encode(&r3[..4]), hex::encode(&fifteen[..4]));
        println!("  3*5==15: {}", if r3 == fifteen { "✅" } else { "❌" });

        // Large value: (2^128) * (2^128) = 2^256 mod p = 38
        let big = {
            let mut v = fe_zero();
            v[16] = 1; // 2^128
            v
        };
        let r4 = fe_mul(&big, &big);
        let thirty_eight = { let mut v = fe_zero(); v[0] = 38; v };
        println!("  (2^128)^2: {} (expect {})", hex::encode(&r4[..4]), hex::encode(&thirty_eight[..4]));
        println!("  (2^128)^2==38: {}", if r4 == thirty_eight { "✅" } else { "❌" });

        // Random-ish large values
        let a_large = {
            let mut v = fe_zero();
            for i in 0..32 { v[i] = (i as u8 * 7 + 13) & 0xFF; }
            v
        };
        let b_large = {
            let mut v = fe_zero();
            for i in 0..32 { v[i] = (i as u8 * 3 + 29) & 0xFF; }
            v
        };
        // Compute a*b mod p using a different method: a*b = a*(b0 + b1*256 + ... + b31*2^248)
        // = sum_i (a * b_i * 256^i)
        // We can compute this as: start with 0, for i in 0..32: result += a*b_i * 2^(8*i)
        // But 2^(8*i) mod p for i >= 32 is 38^(i/32) * 2^(8*(i%32)) mod p
        // This is getting complicated. Let me just check commutativity.
        let r_ab = fe_mul(&a_large, &b_large);
        let r_ba = fe_mul(&b_large, &a_large);
        println!("  a*b:    {}", hex::encode(&r_ab[..8]));
        println!("  b*a:    {}", hex::encode(&r_ba[..8]));
        println!("  a*b==b*a: {}", if r_ab == r_ba { "✅" } else { "❌" });

        // Check fe_sq vs fe_mul(a, a)
        let r_aa = fe_mul(&a_large, &a_large);
        let r_sq = fe_sq(&a_large);
        println!("  a*a:    {}", hex::encode(&r_aa[..8]));
        println!("  fe_sq:  {}", hex::encode(&r_sq[..8]));
        println!("  a*a==sq: {}", if r_aa == r_sq { "✅" } else { "❌" });
    }

    // Test 0g: Large scalar with non-B base
    // k*A should equal (k*scalar mod L)*B since A = scalar*B
    println!("");
    println!("📋 Test 0g: Large scalar cross-check (k*A == (k*scalar mod L)*B)...");
    {
        let seed = [0x42u8; 32];
        let h = Sha512::digest(&seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&h[0..32]);
        scalar[0] &= 0xF8;
        scalar[31] &= 0x7F;
        scalar[31] |= 0x40;
        let a_pt = point_scalar_mul(&scalar, &b, &d, &d2);
        let a_bytes = point_compress(&a_pt);
        let a_dec = point_decompress(&a_bytes, &d).unwrap();
        
        // Large k
        let k_hash = Sha512::digest(b"test k value");
        let mut k_full = [0u8; 64];
        k_full.copy_from_slice(&k_hash);
        let k = sc_reduce_512(&k_full);
        
        // k*A (using decompressed A as base)
        let ka_direct = point_scalar_mul(&k, &a_dec, &d, &d2);
        let ka_direct_c = point_compress(&ka_direct);
        
        // k*scalar mod L
        let ks = sc_mul(&k, &scalar);
        // (k*scalar mod L)*B
        let ka_indirect = point_scalar_mul(&ks, &b, &d, &d2);
        let ka_indirect_c = point_compress(&ka_indirect);
        
        println!("  k*A (direct):   {}", hex::encode(&ka_direct_c[..8]));
        println!("  (k*s)*B (indirect): {}", hex::encode(&ka_indirect_c[..8]));
        println!("  Match: {}", if ka_direct_c == ka_indirect_c { "✅" } else { "❌" });
    }

    // Test 1: Key generation
    println!("");
    println!("📋 Test 1: Génération de clés...");
    let sk = AfriSecretKey::generate();
    let pk = sk.verifying_key();
    println!("  Clé privée: {}", hex::encode(&sk.to_bytes()[..8]));
    println!("  Clé publique: {}", hex::encode(&pk.to_bytes()[..8]));
    println!("  ✅ Clés générées");
    
    // Test 2: Sign and verify
    println!("");
    println!("📋 Test 2: Signature et vérification...");
    let msg = b"Bonjour Afrique!";
    let sig = sk.sign(msg);
    let valid = pk.verify(msg, &sig);
    println!("  Message: {:?}", std::str::from_utf8(msg).unwrap());
    println!("  Signature: {}...", hex::encode(&sig.to_bytes()[..8]));
    println!("  Vérification: {}", if valid { "✅ VALIDE" } else { "❌ INVALIDE" });
    
    if !valid {
        println!("  ❌ ERREUR: La vérification a échoué!");
        return;
    }
    
    // Test 3: Wrong message should fail
    println!("");
    println!("📋 Test 3: Mauvais message...");
    let wrong_msg = b"Mauvais message";
    let valid2 = pk.verify(wrong_msg, &sig);
    println!("  Message: {:?}", std::str::from_utf8(wrong_msg).unwrap());
    println!("  Vérification: {}", if valid2 { "❌ VALIDE (BUG!)" } else { "✅ INVALIDE (correct)" });
    
    if valid2 {
        println!("  ❌ ERREUR: La vérification devrait échouer!");
        return;
    }
    
    // Test 4: Multiple sign/verify
    println!("");
    println!("📋 Test 4: Signatures multiples...");
    let mut all_ok = true;
    for i in 0..5 {
        let m = format!("Test message #{}", i);
        let s = sk.sign(m.as_bytes());
        let v = pk.verify(m.as_bytes(), &s);
        if !v { all_ok = false; }
        println!("  Message {}: {}", i, if v { "✅" } else { "❌" });
    }
    
    println!("");
    if all_ok && valid && !valid2 {
        println!("🎉 AFRI-ED25519 FONCTIONNE! Notre crypto est souveraine! 💚🦁");
    } else {
        println!("❌ Des tests ont échoué...");
    }
}
