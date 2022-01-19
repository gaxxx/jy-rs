use jy::Settings;

#[test]
fn test_settings() {
    let s = Settings::new();
    assert_eq!(s.debug, true);
}