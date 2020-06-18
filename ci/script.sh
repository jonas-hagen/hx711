set -euxo pipefail

main() {
    cargo check --target $TARGET
    case $TARGET in
        x86_64*)
            cargo test
            ;;
        *)
            ;;
    esac
}

main
