mod settings;
mod stats;
mod task;
mod timer;
use pomodorust::app::App;

#[test]
fn test_app_creation() {
    let app = App::new();
    assert_eq!(app.tasks.len(), 0);
}
