set minimum-version := "1.56.0"

set lazy

to_run := env('TO_RUN')

[group("utils")]
cloc-project:
    cloc --vcs git

run_bin:
    cargo run -p {{ to_run }}

run_example_with_hotpath:
    cargo run -p {{ to_run }} -F hotpath,hotpath-cpu,hotpath-alloc

clean:
    cargo clean

hack-clippy:
    cargo hack --each-feature clippy
