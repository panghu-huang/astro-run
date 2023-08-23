fn main() {
  if std::env::var("ASTRO_RUN_PROTOCOL_SKIP_BUILD").is_ok() {
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

  // println!("cargo:rerun-if-changed=proto/astro_run_server.proto");
}
