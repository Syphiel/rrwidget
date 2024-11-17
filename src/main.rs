#![warn(clippy::pedantic)]

mod dbus;
mod structs;
mod www;

use crate::dbus::setup;
use crate::structs::Item;
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
                // TODO: Don't panic on invalid images
                let paintable = gtk::gdk::Texture::from_bytes(&image_bytes).unwrap();
                image = gtk::Image::builder()
                    .paintable(&paintable)
                    .pixel_size(64)
                    .margin_top(5)
                    .margin_bottom(5)
                    .margin_start(5)
                    .margin_end(5)
                    .build();
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
