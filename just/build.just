
# Cleans the build artifacts
[group('build')]
[confirm("Are you sure you want to clean the build artifacts?")]
clean:
  cargo clean


# Builds all artifacts
[group('build')]
build:
  cargo build 
  cargo build --all-features

  # build the test-canister wasm for integration tests
  cargo build -p ic_mple_client_integration_tests --target wasm32-unknown-unknown --release
  cargo build -p ic_mple_scheduler_integration_tests --target wasm32-unknown-unknown --release
  


