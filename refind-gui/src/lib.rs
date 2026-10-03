pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn gui_version_is_available() {
        assert!(!version().is_empty());
    }
}
