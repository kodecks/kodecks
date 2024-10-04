#[cfg(target_family = "wasm")]
fn main() {
    use gloo_worker::Registrable;
    use kodecks_engine::worker::EngineReactor;
    console_error_panic_hook::set_once();
    EngineReactor::registrar().register();
}

#[cfg(not(target_family = "wasm"))]
fn main() {}
