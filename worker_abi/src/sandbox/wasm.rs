#[derive(Debug, Default)]
pub struct WasmSandbox;

impl WasmSandbox {
    pub fn describe(&self) -> &'static str {
        "placeholder wasm sandbox: no syscalls, host-mediated atom access"
    }
}
