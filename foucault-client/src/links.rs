use foucault_server::link_repr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    inner: link_repr::Link,
}

impl Link {
    pub fn new(from: i64, to: String) -> Self {
        Link {
            inner: link_repr::Link { from, to },
        }
    }
}
