//! `sweeploom` CLI and MCP host.

fn main() {
    sweeploom::run_cli(std::env::args().skip(1));
}
