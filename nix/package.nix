{ rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "miniapp-factory-coder";
  version = "1.0.0";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  meta = {
    mainProgram = "miniapp-factory-coder";
  };
}
