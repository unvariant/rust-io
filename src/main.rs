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

        unsafe fn debug <T: std::fmt::Debug> (v: __m128i) {
            let size = std::mem::size_of::<T>();
            let elements = 16 / size;
            let mut array: [u8; 16] = [0; 16];
            _mm_storeu_si128(array.as_ptr() as *mut _, v);
            println!("__m128i.v{}_{}({:?})", elements, size * 8, std::slice::from_raw_parts(array.as_ptr() as *const T, elements));
        }
        
        unsafe fn i32_from_str16_sse(s: &str) -> i32 {
            const DIGITS: Vdata<[u8; 16]> =
                Vdata([0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            const SHIFT: Vdata<[u8; 16]> =
                Vdata([0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7, 0xF6, 0xF5, 0xF4, 0xF3, 0xF2, 0xF1, 0xF0]);

            const M8: Vdata<[u8; 16]> = Vdata([1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10]);
            const M16: Vdata<[u16; 8]> = Vdata([1, 100, 1, 100, 1, 100, 1, 100]);
            const M32: Vdata<[u32; 4]> = Vdata([1, 10000, 1, 10000]);

            let mut raw = s.as_ptr();

            let sign = if *raw == '-' as u8 {
                raw = raw.add(1);
                1
            } else {
                0
            };
            let mask: i32 = 0i32.overflowing_sub(sign).0;

            let bytes = _mm_loadu_si128(raw as *const _);
            let allowed = _mm_load_si128(DIGITS.as_ptr() as *const _);
            let mut shift = _mm_load_si128(SHIFT.as_ptr() as *const _);

            let len = _mm_cmpistri(allowed, bytes,
                _SIDD_UBYTE_OPS | _SIDD_CMP_EQUAL_ANY |
                _SIDD_NEGATIVE_POLARITY | _SIDD_LEAST_SIGNIFICANT);

            shift = _mm_add_epi8(shift, _mm_set1_epi8(len as i8));
            let mut num = _mm_sub_epi8(bytes, _mm_set1_epi8(0x30));
            num = _mm_shuffle_epi8(num, shift);
            num = _mm_maddubs_epi16(num, _mm_load_si128(M8.as_ptr() as *const _));
            num = _mm_madd_epi16(num, _mm_load_si128(M16.as_ptr() as *const _));
            num = _mm_mullo_epi32(num, _mm_load_si128(M32.as_ptr() as *const _));
            num = _mm_hadd_epi32(num, _mm_set1_epi32(0));

            let lo: i32 = _mm_extract_epi32(num, 0);

            let hi: i32 = _mm_extract_epi32(num, 1);
            
            if let Some(n) = hi.checked_mul(100000000) {
                return lo + n;
            } else {
                return (i32::MAX ^ mask) + sign;
            }
        }

    }
}

struct Input {

}

struct Output {

}

fn main() {
    let n = unsafe { i32_from_str16_sse("-4294967295H") };
    println!("{n}");
}
