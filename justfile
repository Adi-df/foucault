default:
    just --list

mock-db-filename := "mock.db"
mock-db-url := "sqlite://" + mock-db-filename + "?mode=rwc"
prepare-queries:
    @echo "Prepare SQLx compile-time queries."
    env DATABASE_URL="{{mock-db-url}}" cargo sqlx migrate run
    env DATABASE_URL="{{mock-db-url}}" cargo sqlx prepare
    rm {{mock-db-filename}}

build-dev:
    cargo build

build-release: prepare-queries
    cargo build --release
