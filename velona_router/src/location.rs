use url::Url;

const VELONA_ROUTER_BASE_URL: &str = "velona:///";

fn get_velona_router_base_url() -> Url {
    Url::parse(VELONA_ROUTER_BASE_URL).unwrap()
}

pub struct Location {
    pub url: Url,
}

#[cfg(test)]
mod tests {
    use super::get_velona_router_base_url;
    #[test]
    fn test_router_base_url() {
        let url = get_velona_router_base_url();

        assert!(url.domain().is_none());
        assert!(!url.origin().is_tuple());
        assert!(url.path_segments().is_none());
    }
}
