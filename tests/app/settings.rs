use pomodorust::{app::App, settings::ColorTheme};

#[test]
fn test_modify_setting() {
    let mut app = App::new();
    app.settings_selection = 3;
    app.modify_setting(true);
    assert_eq!(app.settings.theme, ColorTheme::Dracula);
    app.modify_setting(false);
    assert_eq!(app.settings.theme, ColorTheme::Default);
}
