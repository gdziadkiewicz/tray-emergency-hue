use tray_item::{IconSource, TIError, TrayItem};

//fn setup_tray_app(tx: &mpsc::Sender<Message>) -> Result<TrayItem, TIError> {
pub fn setup_tray_app(quit_cb: impl Fn() + Send + Sync + 'static) -> Result<TrayItem, TIError> {
    let mut tray: TrayItem = TrayItem::new("Tray Example", IconSource::Resource("tray-default"))?;

    //tray.add_label("Tray Label")?;
    //tray.inner_mut().set_tooltip("Piesek")?;

    // let quit_tx = tx.clone();
    // tray.add_menu_item("Quit", move || {
    //     quit_tx.blocking_send(Message::Quit).unwrap();
    // })?;
    tray.add_menu_item("Quit", quit_cb)?;
    Ok(tray)
}

pub enum Icon {
    Red,
    Green,
}

pub fn change_icon(tray_item: &mut TrayItem, icon: &Icon) -> Result<(), TIError> {
    let icon = match icon {
        Icon::Red => "red",
        Icon::Green => "green",
    };
    tray_item.set_icon(IconSource::Resource(icon))
}
