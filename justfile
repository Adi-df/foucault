default:
    just --list

mock-db-filename := "mock.db"
mock-db-filepath := join(justfile_dir(), mock-db-filename)
mock-db-url := "sqlite:///" + trim_start_match(mock-db-filepath, "/") + "?mode=rwc"
prepare-queries:
    @echo "Prepare SQLx compile-time queries."
    cd foucault-server && env DATABASE_URL="{{mock-db-url}}" cargo sqlx migrate run
    cd foucault-server && env DATABASE_URL="{{mock-db-url}}" cargo sqlx prepare
    rm {{mock-db-filepath}}

prepare-dist:
    cargo dist init
    cargo dist plan

build-dev:
    cargo build

build-release: prepare-queries
    cargo build --release
