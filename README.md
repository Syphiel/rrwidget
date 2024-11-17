# RRWidget
A custom waybar reddit widget. Designed to be used in conjunction with sway and waybar to display a notification icon upon new posts in a given subreddit. D-Bus controlled for easy visibility toggling via sway binds.

> [!WARNING]
> This project is designed for my very specific use case and will likely not work for you. If you want to use it, you will need to modify the code to fit your needs.

![Screenshot](screenshot.png)

### Requirements
- [GTK4](https://www.gtk.org/docs/installations/linux)
- [Rust](https://rustup.rs/)

### Installation
Build and install the binary:
```bash
git clone https://github.com/syphiel/RRWidget.git
cd RRWidget
cargo build --release
cp target/release/rrwidget ~/.local/bin/
cp scripts/toggle-widget.sh ~/.local/bin/
```

### Example Usage
Add the following to your waybar config:
```json
"custom/rrwidget": {
    "exec": "GSK_RENDERER=ngl $HOME/.local/bin/rrwidget",
    "tooltip": false,
    "return-type": "json",
    "restart-interval": 60,
    "format": "{icon}",
    "format-icons": {
        "default": "",
        "new": "",
    }
},
```
Enable `custom/rrwidget` in `modules-left`, `modules-center`, or `modules-right` as desired.

Add the following to your sway config:
```
bindsym $mod+u exec /home/montasir/.local/bin/toggle-widget.sh

for_window [app_id="io.syph.rrwidget"] {
    floating enable
    move position center
}
```

### License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
