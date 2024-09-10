#[cfg(target_arch = "wasm32")]
fn main() {
    use gloo_worker::Registrable;
    use kodecks_server::{codec::Json, worker::ServerReactor};
    console_error_panic_hook::set_once();
    ServerReactor::registrar().encoding::<Json>().register();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
