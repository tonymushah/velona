pub use dioxus_devtools_types::DevserverMsg;
use reactive_graph::owner::Owner;

pub fn connect_to_dx_cli<C>(callback: C) -> bool
where
    C: Fn(DevserverMsg) + Send + Sync + 'static,
{
    let Some(endpoint) = dioxus_cli_config::devserver_ws_endpoint() else {
        return false;
    };
    std::thread::spawn(move || {
        let uri = format!(
            "{endpoint}?aslr_reference={}&build_id={}&pid={}",
            subsecond::aslr_reference(),
            dioxus_cli_config::build_id(),
            std::process::id()
        );

        let (mut websocket, _req) = match tungstenite::connect(uri) {
            Ok((websocket, req)) => (websocket, req),
            Err(_) => return,
        };

        while let Ok(msg) = websocket.read() {
            if let tungstenite::Message::Text(text) = msg
                && let Ok(msg) = serde_json::from_str(&text)
            {
                callback(msg);
            }
        }
    });
    true
}

pub fn call<F, O>(fun: F) -> O
where
    F: FnMut() -> O,
{
    if Owner::current().is_none() {
        subsecond::call(fun)
    } else {
        todo!()
    }
}
