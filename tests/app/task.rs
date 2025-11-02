use pomodorust::app::App;

#[test]
fn test_submit_task() {
    let mut app = App::new();
    app.current_input = "test".to_string();
    app.submit_task();
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].name, "test");
}

#[test]
fn test_complete_active_task() {
    let mut app = App::new();
    app.current_input = "test".to_string();
    app.submit_task();
    app.complete_active_task();
    assert!(app.tasks[0].completed);
}

#[test]
fn test_next_previous_task() {
    let mut app = App::new();
    app.current_input = "test1".to_string();
    app.submit_task();
    app.current_input = "test2".to_string();
    app.submit_task();
    app.next_task();
    assert_eq!(app.active_task_index, Some(1));
    app.previous_task();
    assert_eq!(app.active_task_index, Some(0));
}

#[test]
fn test_move_active_task() {
    let mut app = App::new();
    app.current_input = "test1".to_string();
    app.submit_task();
    app.current_input = "test2".to_string();
    app.submit_task();
    app.move_active_task_down();
    assert_eq!(app.active_task_index, Some(1));
    assert_eq!(app.tasks[0].name, "test2");
    app.move_active_task_up();
    assert_eq!(app.active_task_index, Some(0));
    assert_eq!(app.tasks[0].name, "test1");
}
