#![warn(clippy::pedantic)]

mod dbus;
mod structs;
mod www;

use crate::dbus::setup;
use crate::structs::{Config, Item};
use crate::www::get_posts;
use gtk::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, ApplicationWindow};
use lru::LruCache;
use std::io::Write;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("io.syph.rrwidget")
        .build();

    app.connect_activate(|_| {});
    app.connect_startup(buildui);

    app.run()
}

fn buildui(app: &Application) {
    let bx = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let listbox = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    listbox.append(&bx);


    let window = ApplicationWindow::builder()
        .application(app)
        .title("RRWidget")
        .default_width(600)
        .default_height(780)
        .visible(false)
        .build();
    window.set_child(Some(&listbox));

    let (sender, reciever) = async_channel::unbounded();
    let sender_clone = sender.clone();
    let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(10).unwrap())));
    let cache_clone = cache.clone();

    gio::spawn_blocking(move || {
        if let Ok(posts) = get_posts(&mut cache_clone.lock().unwrap()) {
            // Panic if channel fails to send
            sender_clone.send_blocking(posts).unwrap();
        } else {
            eprintln!("Error fetching posts");
        }
    });

    glib::timeout_add_seconds_local(300, move || {
        let sender = sender.clone();
        let cache_clone = cache.clone();
        gio::spawn_blocking(move || {
            if let Ok(posts) = get_posts(&mut cache_clone.lock().unwrap()) {
                // Panic if channel fails to send
                sender.send_blocking(posts).unwrap();
            } else {
                eprintln!("Error fetching posts");
            }
        });
        glib::ControlFlow::Continue
    });

    let new_posts = std::rc::Rc::new(std::cell::Cell::new(false));

    glib::spawn_future_local(populate_list(listbox.clone(), new_posts.clone(), reciever));
    glib::spawn_future_local(setup(window.clone(), new_posts.clone()));
    check_config(&window, app);
}

async fn populate_list(
    listbox: gtk::ListBox,
    new_posts: std::rc::Rc<std::cell::Cell<bool>>,
    reciever: async_channel::Receiver<Vec<Item>>,
) {
    let mut last_seen = String::new();
    while let Ok(posts) = reciever.recv().await {
        listbox.remove_all();
        if !posts.is_empty() && posts[0].id != last_seen {
            last_seen.clone_from(&posts[0].id);
            new_posts.set(true);
            println!("{{\"text\":\"\", \"alt\":\"new\"}}");
            std::io::stdout().flush().unwrap();
        }
        for post in posts {
            let image;
            if let Some(image_data) = post.image_data {
                let image_bytes = glib::Bytes::from_owned(image_data);
                match gtk::gdk::Texture::from_bytes(&image_bytes) {
                    Ok(paintable) => {
                        image = gtk::Image::builder()
                            .paintable(&paintable)
                            .pixel_size(64)
                            .margin_top(5)
                            .margin_bottom(5)
                            .margin_start(5)
                            .margin_end(5)
                            .build();
                    }
                    Err(_) => {
                        image = gtk::Image::builder()
                            .pixel_size(64)
                            .margin_top(5)
                            .margin_bottom(5)
                            .margin_start(5)
                            .margin_end(5)
                            .build();
                    }
                }
            } else {
                image = gtk::Image::builder()
                    .pixel_size(64)
                    .margin_top(5)
                    .margin_bottom(5)
                    .margin_start(5)
                    .margin_end(5)
                    .build();
            }

            let label = gtk::Label::builder()
                .use_markup(true)
                .ellipsize(gtk::pango::EllipsizeMode::Middle)
                .valign(gtk::Align::Center)
                .label(format!("<b>{}</b>\n{}", post.title, post.created))
                .build();

            let bx = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .hexpand(true)
                .build();

            let gesture = gtk::GestureClick::new();

            gesture.connect_released(move |gesture, _, _, _| {
                gio::AppInfo::launch_default_for_uri(
                    &post.url,
                    Option::<&gio::AppLaunchContext>::None,
                )
                .unwrap();
                gesture.set_state(gtk::EventSequenceState::Claimed);
            });
            bx.add_controller(gesture);
            bx.append(&image);
            bx.append(&label);
            listbox.append(&bx);
        }
    }
}

fn check_config(window: &ApplicationWindow, app: &Application) {
    let conf: Config = confy::load("rrwidget", None).expect("Failed to load config");

    if conf.is_valid() {
        return
    }

    let dialog = gtk::Window::builder()
        .title("Unable to load config")
        .application(app)
        .modal(true)
        .transient_for(window)
        .build();

    let ok_button = gtk::Button::builder()
        .label("OK")
        .hexpand(true)
        .build();

    let cancel_button = gtk::Button::builder()
        .label("Cancel")
        .hexpand(true)
        .css_classes(vec!["destructive-action"])
        .build();

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let label = gtk::Label::builder()
        .label("Unable to load config, please configure the following fields:")
        .margin_start(5)
        .margin_end(5)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    let client_id = gtk::Entry::builder()
        .placeholder_text("Client ID")
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .margin_top(5)
        .build();

    let client_secret = gtk::Entry::builder()
        .placeholder_text("Client Secret")
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .build();

    let reddit_user = gtk::Entry::builder()
        .placeholder_text("Reddit User")
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .build();

    let reddit_pass = gtk::Entry::builder()
        .placeholder_text("Reddit Password")
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .build();

    let subreddit = gtk::Entry::builder()
        .placeholder_text("Subreddit")
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .margin_bottom(5)
        .build();

    vbox.append(&label);
    vbox.append(&client_id);
    vbox.append(&client_secret);
    vbox.append(&reddit_user);
    vbox.append(&reddit_pass);
    vbox.append(&subreddit);
    hbox.append(&ok_button);
    hbox.append(&cancel_button);
    vbox.append(&hbox);
    dialog.set_child(Some(&vbox));

    let dialog_clone = dialog.clone();
    ok_button.connect_clicked(move |_| {
        let conf = Config {
            client_id: client_id.text().to_string(),
            client_secret: client_secret.text().to_string(),
            reddit_user: reddit_user.text().to_string(),
            reddit_pass: reddit_pass.text().to_string(),
            subreddit: subreddit.text().to_string(),
        };
        confy::store("rrwidget", None, conf).expect("Failed to store config");
        dialog_clone.close();
    });

    let app = app.clone();
    cancel_button.connect_clicked(move |_| {
        app.quit();
    });

    dialog.present();
}
