use foucault_server::link_repr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    inner: link_repr::Link,
}

impl From<link_repr::Link> for Link {
    fn from(inner: link_repr::Link) -> Self {
        Self { inner }
    }
}

impl Link {
    pub fn new(from: i64, to: String) -> Self {
        Link {
            inner: link_repr::Link { from, to },
        }
    }

    pub fn get_inner(&self) -> &link_repr::Link {
        &self.inner
    }
}
