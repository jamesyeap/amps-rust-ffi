#[cfg(test)]
mod tests {
    #[test]
    fn project_setup_compiles() {
        // Verify that core dependencies are available
        let _: log::Level = log::Level::Info;
        let _: libc::c_int = 0;
    }
}
