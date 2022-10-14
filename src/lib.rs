#[macro_export]
macro_rules! cfg_if {
    // match if/else chains with a final `else`
    ($(
        if #[cfg($meta:meta)] { $($tokens:tt)* }
    ) else * else {
        $($tokens2:tt)*
    }) => {
        $crate::cfg_if! {
            @__items
            () ;
            $( ( ($meta) ($($tokens)*) ), )*
            ( () ($($tokens2)*) ),
        }
    };

    // match if/else chains lacking a final `else`
    (
        if #[cfg($i_met:meta)] { $($i_tokens:tt)* }
        $(
            else if #[cfg($e_met:meta)] { $($e_tokens:tt)* }
        )*
    ) => {
        $crate::cfg_if! {
            @__items
            () ;
            ( ($i_met) ($($i_tokens)*) ),
            $( ( ($e_met) ($($e_tokens)*) ), )*
            ( () () ),
        }
    };

    // Internal and recursive macro to emit all the items
    //
    // Collects all the negated cfgs in a list at the beginning and after the
    // semicolon is all the remaining items
    (@__items ($($not:meta,)*) ; ) => {};
    (@__items ($($not:meta,)*) ; ( ($($m:meta),*) ($($tokens:tt)*) ), $($rest:tt)*) => {
        // Emit all items within one block, applying an appropriate #[cfg]. The
        // #[cfg] will require all `$m` matchers specified and must also negate
        // all previous matchers.
        #[cfg(all($($m,)* not(any($($not),*))))] $crate::cfg_if! { @__identity $($tokens)* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$m` matchers to the list of `$not` matchers as future emissions
        // will have to negate everything we just matched as well.
        $crate::cfg_if! { @__items ($($not,)* $($m,)*) ; $($rest)* }
    };

    // Internal macro to make __apply work out right for different match types,
    // because of how macros matching/expand stuff.
    (@__identity $($tokens:tt)*) => {
        $($tokens)*
    };
}

use core::arch::asm;

#[link(name = "simd")]
extern "C" {
    fn asm_i32_from_str16_sse (s: &[u8]) -> i32;
}

cfg_if! {
    if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature="sse",
        )
    )] {

        use core::arch::x86_64::*;

        #[repr(align(16))]
        #[derive(Debug)]
        struct Vdata<T>(T);

        impl<T> Vdata<T> {
            fn as_ptr (self) -> *const T {
                &self.0 as *const T
            }
        }

        #[allow(dead_code)]
        unsafe fn debug256 <T: std::fmt::Debug> (v: __m256i) {
            let size = std::mem::size_of::<T>();
            let elements = 32 / size;
            let array: [u8; 32] = [0; 32];
            _mm256_storeu_si256(array.as_ptr() as *mut _, v);
            println!("__m256i.v{}_{}({:?})", elements, size * 8, std::slice::from_raw_parts(array.as_ptr() as *const T, elements));
        }

        #[allow(dead_code)]
        unsafe fn debug128 <T: std::fmt::Debug> (v: __m128i) {
            let size = std::mem::size_of::<T>();
            let elements = 16 / size;
            let array: [u8; 16] = [0; 16];
            _mm_storeu_si128(array.as_ptr() as *mut _, v);
            println!("__m256i.v{}_{}({:?})", elements, size * 8, std::slice::from_raw_parts(array.as_ptr() as *const T, elements));
        }

        unsafe fn consume_leading_zeros_sse (s: &[u8]) -> usize {
            let mut offset = 0;

            // scalar loop to make sure the simd loads do not
            // go past the end of the string a potentially segfault
            for i in 0..(s.len() & 0x0F) {
                if s[i] != '0' as u8 {
                    return offset;
                }
                offset += 1;
            }

            let find = _mm_insert_epi8(_mm_setzero_si128(), 0x30, 0);
            let mut len = std::cmp::min(16, s.len() - offset);

            while len == 16 && offset < s.len() {
                let bytes = _mm_loadu_si128(s.as_ptr().add(offset) as *const _);
                len = _mm_cmpistri(find, bytes,
                    _SIDD_UBYTE_OPS | _SIDD_CMP_EQUAL_ANY |
                    _SIDD_NEGATIVE_POLARITY | _SIDD_LEAST_SIGNIFICANT) as usize;

                offset += len;
            }

            offset
        }
        
        // _mm_cmpistri requires sse4.2
        #[inline]
        pub unsafe fn n32_from_str16_sse (mut s: &[u8]) -> (bool, u32, u32) {
            const DIGITS: Vdata<[u8; 16]> =
                Vdata([0x30, 0x39, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            const SHIFT: Vdata<[u8; 16]> =
                Vdata([0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7, 0xF6, 0xF5, 0xF4, 0xF3, 0xF2, 0xF1, 0xF0]);

            const M1: Vdata<[u8; 16]> = Vdata([1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10]);
            const M2: Vdata<[u16; 8]> = Vdata([1, 100, 1, 100, 1, 100, 1, 100]);
            const M4: Vdata<[u16; 8]> = Vdata([1, 10000, 1, 10000, 1, 10000, 1, 10000]);

            let sign = s[0] == '-' as u8;

            s = &s[(sign as usize)..s.len()];

            let leading_zeros = consume_leading_zeros_sse(s);

            s = &s[leading_zeros..s.len()];

            let bytes = _mm_loadu_si128(s.as_ptr() as *const _);
            let allowed = _mm_load_si128(DIGITS.as_ptr() as *const _);
            let mut shift = _mm_load_si128(SHIFT.as_ptr() as *const _);

            let len = std::cmp::min(s.len(), _mm_cmpistri(allowed, bytes,
                _SIDD_UBYTE_OPS | _SIDD_CMP_RANGES |
                _SIDD_NEGATIVE_POLARITY | _SIDD_LEAST_SIGNIFICANT) as usize);

            let mut num = _mm_and_si128(bytes, _mm_set1_epi8(0x0F));
            shift = _mm_add_epi8(shift, _mm_set1_epi8(len as i8));
            num = _mm_shuffle_epi8(num, shift);
            num = _mm_maddubs_epi16(num, _mm_load_si128(M1.as_ptr() as *const _));
            num = _mm_madd_epi16(num, _mm_load_si128(M2.as_ptr() as *const _));
            num = _mm_packus_epi32(num, num);
            num = _mm_madd_epi16(num, _mm_load_si128(M4.as_ptr() as *const _));

            let lo = _mm_extract_epi32(num, 0) as u32;
            let hi = _mm_extract_epi32(num, 1) as u32;

            (sign, lo, hi)
        }

        #[inline]
        pub unsafe fn i32_from_str16_sse (s: &[u8]) -> i32 {
            let (sign, lo, hi) = n32_from_str16_sse(s);
            let sign = sign as i32;
            let lo = lo as i32;
            let hi = hi as i32;
            let mask = 0i32.overflowing_sub(sign as i32).0;
            (if let Some(n) = hi.checked_mul(100000000) {
                lo + n
            } else {
                i32::MAX
            } ^ mask) + sign
        }

        unsafe fn u32_from_str16_sse (s: &[u8]) -> u32 {
            let (sign, lo, hi) = n32_from_str16_sse(s);
            let mask = !0u32.overflowing_sub(sign as u32).0;
            (if let Some(n) = hi.checked_mul(100000000) {
                lo + n
            } else {
                u32::MAX
            }) & mask
        }

        #[inline]
        unsafe fn consume_leading_zeros_avx (s: &[u8]) -> usize {
            let mut offset = 0;
            let mut len = 32;
            let cmpeq = _mm256_set1_epi8(0x30);

            while len == 32 && offset < s.len() {
                let bytes = _mm256_loadu_si256(s[offset..s.len()].as_ptr() as *const _);
                let equal = _mm256_cmpeq_epi8(bytes, cmpeq);
                len = (!_mm256_movemask_epi8(equal)).trailing_zeros();
                offset += len as usize;
            }

            std::cmp::min(offset, s.len())
        }

        // requires avx2
        // unfortunately pcmpistri was not promoted to 256 bits
        #[inline]
        pub unsafe fn n64_from_str32_avx (mut s: &[u8]) -> u64 {
            const MASK: Vdata<[u8; 32]> = Vdata([
                0x00, 0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, 0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2, 0xf1, 0xf0, 0xef, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1
            ]);

            const SHIFT: Vdata<[u8; 32]> = Vdata([
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            ]);

            let shift = _mm256_loadu_si256(SHIFT.as_ptr() as *const _);

            let sign = s[0] == '-' as u8;

            s = &s[(sign as usize)..s.len()];

            let leading_zeros = consume_leading_zeros_avx(s);

            s = &s[leading_zeros..s.len()];

            let bytes = _mm256_loadu_si256(s.as_ptr() as *const _);
            let gt = _mm256_cmpgt_epi8(bytes, _mm256_set1_epi8(0x2F));
            let invgt = _mm256_cmpgt_epi8(bytes, _mm256_set1_epi8(0x39));
            let digit_mask = _mm256_xor_si256(gt, invgt);

            let mask = !_mm256_movemask_epi8(digit_mask);

            let len = std::cmp::min(mask.trailing_zeros(), s.len() as u32);

            let mask = _mm256_add_epi8(_mm256_loadu_si256(MASK.as_ptr() as *const _),
                _mm256_set1_epi8(len as i8));

            let mask = _mm256_cmpgt_epi8(mask, _mm256_setzero_si256());

            let digits = _mm256_and_si256(bytes, mask);

            let shl = _mm256_sub_epi8(shift, _mm256_set1_epi8(32 - len as i8));
            let shr = _mm256_add_epi8(shift, _mm256_set1_epi8(0x60 + len as i8));
            let perm = _mm256_permute2x128_si256(digits, digits, 8);
            let a = _mm256_shuffle_epi8(perm, shr);
            let b = _mm256_shuffle_epi8(digits, shl);
            let mut num = _mm256_or_si256(a, b);

            num = _mm256_and_si256(num, _mm256_set1_epi8(0x0F));
            num = _mm256_maddubs_epi16(num, _mm256_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10));
            num = _mm256_madd_epi16(num, _mm256_set_epi16(1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100));
            num = _mm256_mullo_epi32(num, _mm256_set_epi32(1, 10000, 1, 10000, 1, 10000, 1, 10000));
            num = _mm256_hadd_epi32(num, _mm256_setzero_si256());
            let big = _mm256_mul_epu32(num, _mm256_set_epi32(1, 100000000, 1, 100000000, 1, 100000000, 1, 100000000));
            num = _mm256_srli_epi64(num, 32);
            num = _mm256_add_epi64(big, num);

            let upper = _mm256_extracti128_si256::<1>(num);

            let lo = _mm_extract_epi64(upper, 0) as u64;
            let hi = _mm256_extract_epi64(num, 0) as u64;

            hi * 10000_0000_0000_0000 + lo
        }
    }
}