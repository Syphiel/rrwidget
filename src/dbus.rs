use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use std::io::Write;
use zbus::{connection::Builder, interface};

pub enum Messages {
    Show,
    Hide,
}

struct DbusInterface {
    channel: async_channel::Sender<Messages>,
    visible: bool,
}

#[interface(name = "io.syph.rrwidget1")]
impl DbusInterface {
    async fn toggle_visibility(&mut self) -> zbus::fdo::Result<()> {
        self.set_visible(!self.visible).await
    }

    #[allow(clippy::unused_async)]
    #[zbus(property)]
    async fn visible(&self) -> bool {
        self.visible
    }

    #[allow(clippy::unused_async)]
    #[zbus(property)]
    async fn set_visible(&mut self, value: bool) -> zbus::fdo::Result<()> {
        if value {
            self.channel.send_blocking(Messages::Show).unwrap();
        } else {
            self.channel.send_blocking(Messages::Hide).unwrap();
        }

        self.visible = value;
        Ok(())
    }
}

pub async fn create_dbus_connection(
    sender: async_channel::Sender<Messages>,
) -> zbus::Result<zbus::Connection> {
    let dbus_interface = DbusInterface {
        channel: sender,
        visible: false,
    };
    Builder::session()?
        .name("io.syph.rrwidget1")?
        .serve_at("/io/syph/rrwidget1", dbus_interface)?
        .build()
        .await
}

pub async fn setup(window: ApplicationWindow, new_posts: std::rc::Rc<std::cell::Cell<bool>>) {
    let (sender, reciever) = async_channel::unbounded();
    let mut connection;
    while {
        connection = create_dbus_connection(sender.clone()).await;
        connection.is_err()
    } {
        eprintln!("Failed to create bus, retying in 5 seconds");
        glib::timeout_future_seconds(5).await;
    }
    let connection = connection.unwrap();
    window.connect_close_request(move |_| {
        glib::spawn_future_local(clone!(
            #[strong]
            connection,
            async move {
                connection
                    .object_server()
                    .interface::<_, DbusInterface>("/io/syph/rrwidget1")
                    .await
                    .unwrap()
                    .get_mut()
                    .await
                    .set_visible(false)
                    .await
                    .unwrap();
            }
        ));
        glib::signal::Propagation::Stop
    });
    while let Ok(message) = reciever.recv().await {
        match message {
            Messages::Show => {
                window.set_visible(true);
                new_posts.set(false);
                println!("{{\"text\":\"\", \"alt\":\"default\"}}");
                std::io::stdout().flush().unwrap();
            }
            Messages::Hide => {
                window.set_visible(false);
            }
        }
    }
}
