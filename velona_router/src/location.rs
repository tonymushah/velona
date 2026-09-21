use url::Url;

const VELONA_ROUTER_BASE_URL: &str = "velona:///";

fn get_velona_router_base_url() -> Url {
    Url::parse(VELONA_ROUTER_BASE_URL).unwrap()
}

#[derive(Debug, Clone)]
pub(crate) struct LocationState {
    pub(crate) url: Url,
}

impl Default for LocationState {
    fn default() -> Self {
        Self {
            url: get_velona_router_base_url(),
        }
    }
}

impl LocationState {
    pub fn goto(&mut self, path: &str) -> Result<(), url::ParseError> {
        self.url = self.url.join(path)?;
        Ok(())
    }
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
