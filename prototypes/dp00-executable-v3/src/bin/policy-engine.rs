use std::io;
use via_dp00_executable_v3::{policy_decision, serve};

fn main() -> io::Result<()> {
    serve(|request| policy_decision(&request))
}
