fn main() {
  let build = std::env::var("ASTRO_RUN_PROTOCOL_BUILD")
    .map(|e| e == "true")
    .unwrap_or(false);

  if !build {
    return;
  }

  tonic_build::configure()
    .out_dir("src/pb")
    .compile(
      &[
        "proto/astro_run_server.proto",
        "proto/astro_run_remote_runner.proto",
      ],
      &["proto"],
    )
    .unwrap();
}
