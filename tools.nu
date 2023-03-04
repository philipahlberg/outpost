def print-spaced [content: string] {
    ""
    $content
    ""
}

# Install the tools necessary for working with the project
export def setup [] {
    print-spaced "> cargo --version"
    cargo --version

    print-spaced "> cargo install cargo-audit --version ^0.17 --locked --features vendored-openssl"
	cargo install cargo-audit --version ^0.17 --locked --features vendored-openssl

    print-spaced "> cargo install cargo-udeps --version ^0.1 --locked --features vendored-openssl"
	cargo install cargo-udeps --version ^0.1 --locked --features vendored-openssl

    print-spaced "> cargo install cargo-outdated --version ^0.11 --locked --features vendored-openssl"
	cargo install cargo-outdated --version ^0.11 --locked --features vendored-openssl
}

# Check formatting, code quality, available updates
export def check [] {
    print-spaced "> cargo fmt -v --all -- --check"
    cargo fmt -v --all -- --check

    print-spaced "> cargo clippy --all-features -- -D warnings"
    cargo clippy --all-features -- -D warnings

    print-spaced "> cargo audit"
    cargo audit

    print-spaced "> cargo +nightly udeps"
    cargo +nightly udeps

    print-spaced "> cargo outdated --exit-code 1"
    cargo outdated --exit-code 1
}
