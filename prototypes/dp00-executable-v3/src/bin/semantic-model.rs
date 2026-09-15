use std::io;
use via_dp00_executable_v3::{semantic_result, serve};

fn main() -> io::Result<()> {
    serve(|request| semantic_result(&request))
}
