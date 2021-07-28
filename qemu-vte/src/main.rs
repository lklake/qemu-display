use glib::{clone, MainContext};
use qemu_display_listener::Chardev;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use vte::prelude::*;
use vte::{glib, gtk, gio};
use zbus::azync::Connection;
use futures::prelude::*;

fn main() {
    pretty_env_logger::init();

    let app = gtk::Application::new(Some("org.qemu.vte-example"), Default::default());
    app.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title(Some("D-Bus serial example"));
        window.set_default_size(350, 70);

        let term = vte::Terminal::new();

        window.set_child(Some(&term));

        MainContext::default().spawn_local(clone!(@strong window => async move {
            let conn = Connection::new_session().await
                .expect("Failed to connect to DBus")
                .into();

            if let Ok(c) = Chardev::new(&conn, "serial").await {
                let (p0, p1) = UnixStream::pair().unwrap();
                if c.proxy.register(p1.as_raw_fd().into()).await.is_ok() {
                    log::info!("ok");
                    let istream = unsafe { gio::UnixInputStream::take_fd(p0) }
                        .dynamic_cast::<gio::PollableInputStream>()
                        .unwrap();
                    let mut read = istream.into_async_read().unwrap();
                    loop {
                        let mut buffer = [0u8; 8192];
                        match read.read(&mut buffer[..]).await {
                            Ok(0) => break,
                            Ok(len) => {
                                term.feed(&buffer[..len]);
                            }
                            Err(e) => {
                                log::warn!("{}", e);
                                break;
                            }
                        }
                    }
                }
            }
        }));

        window.show();
    });

    app.run();
}