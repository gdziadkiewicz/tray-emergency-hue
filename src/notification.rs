pub fn show_notification(summary: &str, body: &str) {
    use notify_rust::Notification;
    Notification::new()
        .summary(summary)
        .body(body)
        .show()
        .unwrap();
}
