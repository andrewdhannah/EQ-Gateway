fn main() {
    uniffi::generate_scaffolding_for_crate("src/eq_ffi.udl", "eq_ffi").unwrap();
}
