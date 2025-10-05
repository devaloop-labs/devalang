#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    devalang_wasm::tools::cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("CLI feature is disabled; rebuild with --features cli");
}
