use std::collections::HashMap;

use crate::utils::HandlerId;

pub type NoParamHandler = Box<dyn Fn() + Send>;

pub type EventMap<T> = HashMap<HandlerId, T>;
