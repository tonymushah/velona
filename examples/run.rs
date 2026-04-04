fn main() {
    env_logger::init();
    velona::app::Builder::default().run().unwrap()
}
