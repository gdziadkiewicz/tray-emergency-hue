use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

enum Message {
    Quit,
    Green,
    Red,
}

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    use log::LevelFilter;
    use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
    use std::fs::File;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("my_rust_binary.log")?,
        ),
    ])?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;
    let mut tray = TrayItem::new(
        "Tray Example",
        IconSource::Resource("name-of-icon-in-rc-file"),
    )
    .unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        log::info!("Hello!");
    })
    .unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let red_tx = tx.clone();
    tray.add_menu_item("Red", move || {
        red_tx.send(Message::Red).unwrap();
    })?;

    let green_tx = tx.clone();
    tray.add_menu_item("Green", move || {
        green_tx.send(Message::Green).unwrap();
    })?;

    tray.inner_mut().add_separator()?;

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })?;

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                log::info!("Quit");
                break;
            }
            Ok(Message::Red) => {
                log::info!("Red");
                use notify_rust::Notification;
                Notification::new()
                    .appname("Piesek")
                    //.sound_name(name)
                    .summary("ALARM!")
                    .body("Paulina Cię wzywa!!!")
                    //.icon("D:\\repos\\tray\\tray-emergency-hue\\icons\\icon-red.ico")
                    .show()?;
                tray.set_icon(IconSource::Resource("another-name-from-rc-file"))?
            }
            Ok(Message::Green) => {
                log::info!("Green");
                tray.set_icon(IconSource::Resource("name-of-icon-in-rc-file"))?
            }
            _ => {}
        }
    }
    Ok(())
}
