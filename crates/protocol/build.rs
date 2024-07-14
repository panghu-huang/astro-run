use proto::proto;

fn main() {
  let remote_runner_service = proto! {
    package remote_runner;
    codec crate::common::JsonCodec;

    service RemoteRunner {
      rpc GetRunnerMetadata(crate::Empty) returns (astro_run_scheduler::RunnerMetadata) {}
      rpc Run(astro_run::Context) returns (stream crate::RunResponse) {}
      rpc SendEvent(crate::RunEvent) returns (crate::Empty) {}
      rpc CallBeforeRunStepHook(astro_run::Command) returns (astro_run::Command) {}
    }
  };

  tonic_build::manual::Builder::new()
    .out_dir("./src/proto")
    .compile(&[remote_runner_service])
}
