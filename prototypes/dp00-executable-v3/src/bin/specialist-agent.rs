use std::io;
use via_dp00_executable_v3::{agent_result, serve};

fn main() -> io::Result<()> {
    serve(|request| agent_result(request, "specialist-agent"))
}
