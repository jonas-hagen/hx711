set -euxo pipefail

main() {
    cargo check --target $TARGET
    cargo test
}

main
