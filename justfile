set minimum-version := "1.56.0"

set lazy

to_run := env('TO_RUN')

features := env('FEATURES', ",")

hot_reload_port := env("HOT_RELOAD_PORT", "8009")

hot_reload_address := env("HOT_RELOAD_ADDRESS", "127.0.0.1")

[group("utils")]
cloc-project:
    cloc --vcs git

run_bin:
    cargo run -p {{ to_run }} -F {{ features }}

run_example_with_hotpath:
    cargo run -p {{ to_run }} -F hotpath,hotpath-cpu,hotpath-alloc

run_hot_patch:
    dx serve -p {{ to_run }} \
        --features {{ features }} \
        --hot-patch

clean:
    cargo clean

hack_clippy:
    cargo hack --each-feature clippy

fmt:
    cargo hack fmt

check_fmt:
    cargo hack fmt --check

run_tests:
    cargo hack --each-feature \
        --exclude align \
        --exclude badge_ed \
        --exclude checkbox \
        --exclude counter \
        --exclude fragment \
        --exclude image_example \
        --exclude todo \
        --exclude todo_softbuffer \
        --exclude todo_subsecond \
        --exclude todo_vello_hybrid \
        test
