// Aggregator test: include tests from tests/rust/* as distinct modules.
// This keeps sources organized while providing a single integration test
// file that Cargo will compile and run.

mod rust_tests {
    pub mod cli_help {
        include!("rust/cli_help.rs");
    }
    pub mod cli_version {
        include!("rust/cli_version.rs");
    }
    pub mod cli_template_list {
        include!("rust/cli_template_list.rs");
    }
    pub mod cli_check_build {
        include!("rust/cli_check_build.rs");
    }
}

// Re-export tests so the test runner finds them at crate root.
pub use rust_tests::*;
