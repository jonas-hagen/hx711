set -euxo pipefail

run_tests() {
    case $TRAVIS_RUST_VERSION in
        stable)
            cargo test
            ;;
        beta)
            cargo test
            ;;
        nightly)
            cargo test --features "never_type"
            ;;
        *)
            ;;
    esac
}

main() {
    cargo check --target $TARGET
    case $TARGET in
        x86_64*)
            run_tests
            ;;
        *)
            ;;
    esac
}

main
