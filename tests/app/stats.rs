use pomodorust::app::App;

#[test]
fn test_delete_selected_completed_task() {
    let mut app = App::new();
    app.current_input = "test1".to_string();
    app.submit_task();
    app.current_input = "test2".to_string();
    app.submit_task();
    app.complete_active_task();
    app.next_task();
    app.complete_active_task();
    app.completed_task_list_state = Some(0);
    app.delete_selected_completed_task();
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].name, "test2");
}
