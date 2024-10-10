init:
    sed -i '' 's#rust-learn-days/template#rust-learn-days/httpie#g' cliff.toml
    git init
    git add .
    git commit -m "init"
    git branch -M main
    git remote add origin git@github.com:rust-learn-days/httpie.git
    pip install pre-commit
    cargo install --locked cargo-deny
    cargo install typos-cli
    cargo install git-cliff
    cargo install cargo-nextest --locked
    pre-commit install

install-cross:
    cargo install cross --git https://github.com/cross-rs/cross

build:
    rustup target add ${target}
    cargo build --target  ${target}

cross-build:
    rustup target add ${target}
    cross build --target  ${target}

cross-build-release:
    rustup target add ${target}
    cross build --release --target  ${target}
    mkdir -p output/${target}
    cp target/${target}/release/httpie output/${target}/httpie
    cd output/${target} && tar -zcvf httpie-${target}.tar.gz httpie

build-release:
    rustup target add ${target}
    cargo build --release --target  ${target}
    mkdir -p output/${target}
    cp target/${target}/release/httpie output/${target}/httpie
    cd output/${target} && tar -zcvf httpie-${target}.tar.gz httpie

changelog:
    git-cliff --config cliff.toml > CHANGELOG.md

line:
    tokei .

check:
    cargo check
    cargo fmt -- --check
    cargo deny check -d
    cargo nextest run --all-features
