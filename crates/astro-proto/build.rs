fn main() {
  tonic_build::configure()
    .out_dir("src/pb")
    .compile(&["proto/astro.proto"], &["proto"])
    .unwrap();

  println!("cargo:rerun-if-changed=proto/astro.proto");
}
