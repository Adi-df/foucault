default:
    just --list

mock-db-filename := "mock.db"
mock-db-filepath := justfile_dir() + "/" + mock-db-filename
mock-db-url := "sqlite://" + mock-db-filepath + "?mode=rwc"
prepare-queries:
    @echo "Prepare SQLx compile-time queries."
    cd foucault-client && env DATABASE_URL="{{mock-db-url}}" cargo sqlx migrate run
    cd foucault-client && env DATABASE_URL="{{mock-db-url}}" cargo sqlx prepare
    rm {{mock-db-filepath}}

build-dev:
    cargo build

build-release: prepare-queries
    cargo build --release
