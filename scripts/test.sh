# cargo-nextest has a problem when dealing with proc-macro crates
# especially to capwat-macros crate.
set -e

script_file_dir_name=$(dirname "$0")
crates_directory="$script_file_dir_name/../crates"

for entry in $crates_directory/*/; do
    if [[ -d "$entry" && ! -L "$entry" ]]; then
        crate_name=${entry%*/}
        crate_name="${crate_name##*/}"

        if [[ "$crate_name" != "capwat-macros" ]]; then
            echo "Testing crate $crate_name..."
            cargo nextest run -p $crate_name
        fi;
    fi;
done

echo "Testing crate src/config"
cargo nextest run -p capwat-config

echo "Testing crate src/model"
cargo nextest run -p capwat-model -j4

echo "Testing crate src/server"
cargo nextest run -p capwat-server -j4
