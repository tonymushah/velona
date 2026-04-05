use log::warn;

pub(crate) fn todo_warn_of_something(something: &'static str) {
    if something.is_empty() {
        warn!("Not yet implemented")
    } else {
        warn!("Not yet implemented {something}")
    }
}

pub(crate) fn todo_warn() {
    todo_warn_of_something("");
}
