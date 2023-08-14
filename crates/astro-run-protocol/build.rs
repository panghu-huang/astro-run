fn main() {
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
