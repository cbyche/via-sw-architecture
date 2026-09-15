use std::io;
use via_dp00_executable_v3::{JsonChild, serve};

fn main() -> io::Result<()> {
    let mut downstream = JsonChild::spawn("general-agent", "general-agent")?;
    serve(move |request| {
        let (mut response, tx, rx) = downstream.request(&request).expect("proxy downstream IPC");
        response["proxy"] = serde_json::json!({"pid":std::process::id(),"child_pid":downstream.pid(),"wire_bytes":tx+rx});
        response
    })
}
