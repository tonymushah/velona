use std::collections::HashMap;

use winit::event::DeviceEvent;

use crate::utils::{
    HandlerId,
    events::{EventMap, NoParamHandler},
};

pub type DeviceEventHandler = Box<dyn Fn(&DeviceEvent) + Send>;

#[derive(Debug)]
pub struct RegisterAppEvent {
    pub handler_id: HandlerId,
    pub type_: RegisterAppEventType,
}

#[derive(derive_more::Debug)]
pub enum RegisterAppEventType {
    Device(#[debug(skip)] DeviceEventHandler),
    MemoryWarning(#[debug(skip)] NoParamHandler),
    Resumed(#[debug(skip)] NoParamHandler),
    Suspended(#[debug(skip)] NoParamHandler),
}

#[derive(Debug)]
pub struct UnRegisterAppEventHandler {
    pub handler_id: HandlerId,
    pub type_: Option<UnRegisterAppEventType>,
}

#[derive(derive_more::Debug)]
pub enum UnRegisterAppEventType {
    Device,
    MemoryWarning,
    Resumed,
    Suspended,
}

#[derive(Default, derive_more::Debug)]
pub struct AppEventHandlers {
    #[debug("HashMap<len = {}>", device.len())]
    device: EventMap<DeviceEventHandler>,
    #[debug("HashMap<len = {}>", memory_warning.len())]
    memory_warning: EventMap<NoParamHandler>,
    #[debug("HashMap<len = {}>", resumed.len())]
    resumed: EventMap<NoParamHandler>,
    #[debug("HashMap<len = {}>", suspended.len())]
    suspended: EventMap<NoParamHandler>,
}

pub enum EmitEventToHandlers<'a> {
    Device(&'a DeviceEvent),
    MemoryWarning,
    Resumed,
    Suspended,
}

impl AppEventHandlers {
    pub fn register_handler(&mut self, handler: RegisterAppEvent) {
        match handler.type_ {
            RegisterAppEventType::Device(h) => {
                self.device.insert(handler.handler_id, h);
            }
            RegisterAppEventType::MemoryWarning(h) => {
                self.memory_warning.insert(handler.handler_id, h);
            }
            RegisterAppEventType::Resumed(h) => {
                self.resumed.insert(handler.handler_id, h);
            }
            RegisterAppEventType::Suspended(h) => {
                self.suspended.insert(handler.handler_id, h);
            }
        }
    }
    fn unregister_handler_from_none(&mut self, handler_id: &HandlerId) {
        macro_rules! unregister {
            ($($field:ident,)*) => {
                $(
                    self.$field.remove(handler_id);
                )*
            };
        }
        unregister!(device, memory_warning, resumed, suspended,);
    }
    pub fn unregister_handler(&mut self, handler: UnRegisterAppEventHandler) {
        let handler_id = handler.handler_id;
        if let Some(_type) = handler.type_ {
            match _type {
                UnRegisterAppEventType::Device => {
                    self.device.remove(&handler_id);
                }
                UnRegisterAppEventType::MemoryWarning => {
                    self.memory_warning.remove(&handler_id);
                }
                UnRegisterAppEventType::Resumed => {
                    self.resumed.remove(&handler_id);
                }
                UnRegisterAppEventType::Suspended => {
                    self.suspended.remove(&handler_id);
                }
            }
        } else {
            self.unregister_handler_from_none(&handler_id);
        }
    }
    pub fn shrink_to_fit(&mut self) {
        macro_rules! impl_shrink_fit {
            ($($field:ident,)*) => {
                $(
                    self.$field.shrink_to_fit();
                )*
            };
        }
        impl_shrink_fit!(device, memory_warning, resumed, suspended,);
    }
    pub fn emit(&self, event: EmitEventToHandlers<'_>) {
        match event {
            EmitEventToHandlers::Device(device_event) => {
                self.device.values().for_each(|h| h(device_event));
            }
            EmitEventToHandlers::MemoryWarning => {
                self.memory_warning.values().for_each(|h| h());
            }
            EmitEventToHandlers::Resumed => {
                self.resumed.values().for_each(|h| h());
            }
            EmitEventToHandlers::Suspended => {
                self.suspended.values().for_each(|h| h());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::event_handlers::AppEventHandlers;

    #[test]
    fn test_app_event_handlers_dbg() {
        println!("{:#?}", AppEventHandlers::default())
    }
}
