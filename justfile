set minimum-version := "1.56.0"

set lazy

to_run := env('TO_RUN')

features := env('FEATURES', ",")

# hot_reload_port := env("HOT_RELOAD_PORT", "8009")

# hot_reload_address := env("HOT_RELOAD_ADDRESS", "127.0.0.1")

cargo_runner := env("CARGO_RUNNER", "cargo")

[group("utils")]
cloc-project:
    cloc --vcs git

run_bin:
    {{ cargo_runner }} run -p {{ to_run }} -F {{ features }}

run_bin_no_default_features:
    {{ cargo_runner }} run -p {{ to_run }} --no-default-features -F {{ features }}

run_example_with_hotpath:
    {{ cargo_runner }} run -p {{ to_run }} -F hotpath,hotpath-cpu,hotpath-alloc

run_hot_patch:
    dx serve -p {{ to_run }} \
        --features {{ features }} \
        --hot-patch

clean:
    cargo clean

hack_clippy:
    cargo hack --each-feature \
        --exclude velona \
        --exclude velona_accesskit_xplat \
        --exclude counter \
        clippy
    cargo hack \
        -p velona \
        -p velona_accesskit_xplat \
        -p counter \
         clippy

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
        --exclude counter \
        --exclude router_example \
        --exclude timer_example \
        --exclude timer-async-io-example \
        --exclude custom_title_bar \
        --exclude velona \
        --exclude velona_accesskit_xplat \
        test
