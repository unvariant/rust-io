fn main() {
    cc::Build::new()
        .file("src/simd.s")
        .compile("simd");
}