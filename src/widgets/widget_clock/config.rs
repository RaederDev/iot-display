pub fn validate(config: &crate::config::Config) -> crate::Result<()> {
    if config.clock_format.is_empty()
        || chrono::format::StrftimeItems::new(&config.clock_format)
            .any(|item| matches!(item, chrono::format::Item::Error))
    {
        return Err("Invalid clock format".into());
    }
    Ok(())
}
