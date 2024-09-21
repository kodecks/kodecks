#[cfg(target_arch = "wasm32")]
fn main() {
    use gloo_worker::Registrable;
    use kodecks_engine::{codec::Json, worker::EngineReactor};
    console_error_panic_hook::set_once();
    EngineReactor::registrar().encoding::<Json>().register();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
