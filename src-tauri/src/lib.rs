use std::sync::mpsc::{self, Receiver, Sender};
use tauri::Manager;

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
enum LauncherCommand {
    ShowCollapsed,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct LauncherController {
    sender: Sender<LauncherCommand>,
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn show_launcher(
    app: tauri::AppHandle,
    state: tauri::State<'_, LauncherController>,
) -> Result<(), String> {
    use tauri::Manager;

    // Nativen GTK-Launcher wieder anzeigen
    state
        .sender
        .send(LauncherCommand::ShowCollapsed)
        .map_err(|error| error.to_string())?;

    // Vollständiges Vue/Tauri-Fenster verstecken
    if let Some(window) =
        app.get_webview_window("main")
    {
        window
            .hide()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(target_os = "linux")]
    let builder = builder.invoke_handler(
        tauri::generate_handler![show_launcher]
    );

    builder
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(target_os = "linux")]
            {
                let (sender, receiver) =
                    mpsc::channel::<LauncherCommand>();

                app.manage(
                    LauncherController { sender }
                );

                create_native_launcher(
                    app,
                    receiver,
                )?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[cfg(target_os = "linux")]
fn open_full_window(
    window: &tauri::WebviewWindow,
) {
    use tauri::LogicalSize;

    let _ = window.set_min_size(
        Some(
            LogicalSize::new(
                900.0,
                650.0,
            )
        )
    );

    let _ =
        window.set_max_size::<LogicalSize<f64>>(
            None
        );

    let _ =
        window.set_resizable(true);

    let _ =
        window.set_decorations(true);

    let _ =
        window.set_always_on_top(false);

    let _ =
        window.set_size(
            LogicalSize::new(
                1150.0,
                800.0,
            )
        );

    let _ =
        window.center();

    let _ =
        window.show();

    let _ =
        window.set_focus();
}


#[cfg(target_os = "linux")]
fn create_native_launcher(
    app: &mut tauri::App,
    receiver: Receiver<LauncherCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::gdk;
    use gtk::glib;
    use gtk::prelude::*;

    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    use tauri::Manager;

    // =========================================================
    // TAURI / VUE HAUPTFENSTER
    // =========================================================

    let notesody_window = app
        .get_webview_window("main")
        .expect("Notesody main window not found");

    // Beim Start ist die Vue-App unsichtbar.
    // Sichtbar ist nur das native blaue Notizbuch.
    notesody_window.hide()?;


    // =========================================================
    // NATIVES GTK-FENSTER
    //
    // Notizbuch und Schnellmenü sind EIN Fenster.
    // Dadurch bewegen sie sich immer gemeinsam.
    // =========================================================

    let launcher =
        gtk::Window::new(
            gtk::WindowType::Toplevel
        );

    launcher.set_title(
        "Bohemian Notesody Launcher"
    );

    launcher.set_decorated(false);
    launcher.set_resizable(false);

    launcher.set_keep_above(true);

    launcher.set_skip_taskbar_hint(true);
    launcher.set_skip_pager_hint(true);

    launcher.set_app_paintable(true);

    launcher.set_default_size(
        92,
        104,
    );


    // =========================================================
    // TRANSPARENZ
    // =========================================================

    if let Some(screen) =
        gtk::prelude::WidgetExt::screen(
            &launcher
        )
    {
        if let Some(visual) =
            screen.rgba_visual()
        {
            launcher.set_visual(
                Some(&visual)
            );
        }
    }


    // =========================================================
    // CSS
    // =========================================================

    let css =
        gtk::CssProvider::new();

    css.load_from_data(
        br#"

        window {
            background-color:
                rgba(0, 0, 0, 0);
        }

        #root {
            background-color:
                rgba(0, 0, 0, 0);
        }

        #book-row {
            background-color:
                rgba(0, 0, 0, 0);

            padding: 6px;
        }

        #book-spine {
            background: #49a6d4;

            border-radius:
                8px 0 0 8px;

            min-width: 13px;
            min-height: 86px;
        }

        #book-button {
            background: #0079b8;
            color: white;

            border: none;

            border-radius:
                0 13px 13px 0;

            min-width: 63px;
            min-height: 86px;

            font-size: 28px;
            font-weight: bold;

            padding: 0;

            box-shadow:
                0 8px 20px
                rgba(0, 0, 0, 0.22);
        }

        #book-button:hover {
            background: #0087c9;
        }

        #book-button:active {
            background: #00689e;
        }

        #menu-shell {
            background: #f6f8fb;

            border-radius: 14px;

            border:
                1px solid #d9e2ec;

            padding: 0;

            margin-top: 0;

            box-shadow:
                0 12px 28px
                rgba(15, 23, 42, 0.18);
        }

        #menu-header {
            background: #ffffff;

            border-radius:
                14px 14px 0 0;

            padding:
                14px 16px;
        }

        #menu-title {
            color: #111827;

            font-weight: 700;
            font-size: 15px;
        }

        #menu-subtitle {
            color: #8190a6;
            font-size: 11px;
        }

        #section {
            background: #ffffff;

            padding:
                14px 16px;
        }

        #section-title {
            color: #111827;

            font-weight: 700;
            font-size: 13px;
        }

        #quick-note {
            min-height: 42px;

            border-radius: 9px;

            border:
                1px solid #d8e1eb;

            padding:
                8px 10px;

            color: #172033;

            background: #fbfcfe;
        }

        #primary-button {
            background: #0079b8;
            color: white;

            border: none;
            border-radius: 9px;

            padding:
                8px 14px;

            font-weight: 700;
        }

        #primary-button:hover {
            background: #006da7;
        }

        #action-button {
            background: #ffffff;
            color: #172033;

            border: none;

            border-top:
                1px solid #e2e8f0;

            border-radius: 0;

            padding:
                12px 16px;

            font-weight: 600;
        }

        #action-button:hover {
            background: #f8fafc;
        }

        #full-button {
            background: #f3f6f9;
            color: #0071ad;

            border: none;

            border-top:
                1px solid #e2e8f0;

            border-radius:
                0 0 14px 14px;

            padding:
                11px 16px;

            font-weight: 700;
        }

        #full-button:hover {
            background: #eaf0f5;
        }

        "#,
    )?;


    if let Some(screen) =
        gdk::Screen::default()
    {
        gtk::StyleContext::
            add_provider_for_screen(
                &screen,
                &css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
    }


    // =========================================================
    // ROOT
    // =========================================================

    let root =
        gtk::Box::new(
            gtk::Orientation::Vertical,
            0,
        );

    root.set_widget_name("root");

    root.set_halign(
        gtk::Align::Center
    );


    // =========================================================
    // NOTIZBUCH
    // =========================================================

    let book_row =
        gtk::Box::new(
            gtk::Orientation::Horizontal,
            0,
        );

    book_row.set_widget_name(
        "book-row"
    );

    book_row.set_halign(
        gtk::Align::Center
    );


    // Buchrücken = Drag-Griff
    let drag_handle =
        gtk::EventBox::new();

    drag_handle.set_widget_name(
        "book-spine"
    );

    drag_handle.set_size_request(
        13,
        86,
    );

    drag_handle.add_events(
        gdk::EventMask::BUTTON_PRESS_MASK
    );


    // Buchfläche = Schnellmenü ein / aus
    let book_button =
        gtk::Button::with_label("☰");

    book_button.set_widget_name(
        "book-button"
    );

    book_button.set_size_request(
        63,
        86,
    );

    book_button.set_tooltip_text(
        Some("Notesody Schnellmenü")
    );


    book_row.pack_start(
        &drag_handle,
        false,
        false,
        0,
    );

    book_row.pack_start(
        &book_button,
        false,
        false,
        0,
    );


    // =========================================================
    // SCHNELLMENÜ
    // =========================================================

    let revealer =
        gtk::Revealer::new();

    revealer.set_transition_type(
        gtk::RevealerTransitionType::SlideDown
    );

    revealer.set_transition_duration(
        160
    );

    revealer.set_reveal_child(false);


    let menu_shell =
        gtk::Box::new(
            gtk::Orientation::Vertical,
            0,
        );

    menu_shell.set_widget_name(
        "menu-shell"
    );

    menu_shell.set_size_request(
        350,
        -1,
    );


    // Header
    let menu_header =
        gtk::Box::new(
            gtk::Orientation::Vertical,
            2,
        );

    menu_header.set_widget_name(
        "menu-header"
    );


    let title =
        gtk::Label::new(
            Some("Bohemian Notesody")
        );

    title.set_widget_name(
        "menu-title"
    );

    title.set_xalign(0.0);


    let subtitle =
        gtk::Label::new(
            Some("Schnellzugriff")
        );

    subtitle.set_widget_name(
        "menu-subtitle"
    );

    subtitle.set_xalign(0.0);


    menu_header.pack_start(
        &title,
        false,
        false,
        0,
    );

    menu_header.pack_start(
        &subtitle,
        false,
        false,
        0,
    );


    menu_shell.pack_start(
        &menu_header,
        false,
        false,
        0,
    );


    // Schnellnotiz
    let note_section =
        gtk::Box::new(
            gtk::Orientation::Vertical,
            8,
        );

    note_section.set_widget_name(
        "section"
    );


    let note_title =
        gtk::Label::new(
            Some("Schnellnotiz")
        );

    note_title.set_widget_name(
        "section-title"
    );

    note_title.set_xalign(0.0);


    let quick_note =
        gtk::Entry::new();

    quick_note.set_widget_name(
        "quick-note"
    );

    quick_note.set_placeholder_text(
        Some(
            "Was möchtest du festhalten?"
        )
    );


    let save_button =
        gtk::Button::with_label(
            "Speichern"
        );

    save_button.set_widget_name(
        "primary-button"
    );

    save_button.set_halign(
        gtk::Align::End
    );


    note_section.pack_start(
        &note_title,
        false,
        false,
        0,
    );

    note_section.pack_start(
        &quick_note,
        false,
        false,
        0,
    );

    note_section.pack_start(
        &save_button,
        false,
        false,
        0,
    );


    menu_shell.pack_start(
        &note_section,
        false,
        false,
        0,
    );


    // Aufgabe
    let task_button =
        gtk::Button::with_label(
            "✓   Neue Aufgabe"
        );

    task_button.set_widget_name(
        "action-button"
    );


    menu_shell.pack_start(
        &task_button,
        false,
        false,
        0,
    );


    // Prozess
    let process_button =
        gtk::Button::with_label(
            "◇   Neuer Prozess"
        );

    process_button.set_widget_name(
        "action-button"
    );


    menu_shell.pack_start(
        &process_button,
        false,
        false,
        0,
    );


    // Vollständig öffnen
    let full_button =
        gtk::Button::with_label(
            "↗   Vollständig öffnen"
        );

    full_button.set_widget_name(
        "full-button"
    );


    menu_shell.pack_start(
        &full_button,
        false,
        false,
        0,
    );


    revealer.add(
        &menu_shell
    );


    // =========================================================
    // ROOT ZUSAMMENBAUEN
    // =========================================================

    root.pack_start(
        &book_row,
        false,
        false,
        0,
    );

    root.pack_start(
        &revealer,
        false,
        false,
        0,
    );


    launcher.add(
        &root
    );


    // =========================================================
    // VERSCHIEBEN
    // =========================================================

    let launcher_for_drag =
        launcher.clone();

    drag_handle.connect_button_press_event(
        move |_, event| {

            if event.button() == 1 {

                if let Some(gdk_window) =
                    launcher_for_drag.window()
                {
                    let (x, y) =
                        event.root();

                    gdk_window.begin_move_drag(
                        1,
                        x as i32,
                        y as i32,
                        event.time(),
                    );
                }
            }

            glib::Propagation::Stop
        },
    );


    // =========================================================
    // MENÜSTATUS
    // =========================================================

    let menu_open =
        Rc::new(
            Cell::new(false)
        );


    // =========================================================
    // NOTIZBUCH KLICK:
    // Schnellmenü ein / aus
    // =========================================================

    let menu_open_for_click =
        menu_open.clone();

    let revealer_for_click =
        revealer.clone();

    let launcher_for_click =
        launcher.clone();


    book_button.connect_clicked(
        move |_| {

            let open =
                !menu_open_for_click.get();

            menu_open_for_click.set(
                open
            );

            revealer_for_click
                .set_reveal_child(
                    open
                );


            if open {

                launcher_for_click.resize(
                    370,
                    500,
                );

            } else {

                launcher_for_click.resize(
                    92,
                    104,
                );
            }
        },
    );


    // =========================================================
    // SCHNELLNOTIZ
    // =========================================================

    let quick_note_for_save =
        quick_note.clone();

    save_button.connect_clicked(
        move |_| {

            let text =
                quick_note_for_save
                    .text()
                    .to_string();

            let clean =
                text.trim();

            if clean.is_empty() {
                return;
            }

            println!(
                "Notesody Schnellnotiz: {}",
                clean
            );

            quick_note_for_save
                .set_text("");
        },
    );


    let save_button_for_enter =
        save_button.clone();

    quick_note.connect_activate(
        move |_| {
            save_button_for_enter
                .emit_clicked();
        },
    );


    // =========================================================
    // HILFSFUNKTION FÜR NATIVE BUTTONS:
    // Launcher weg -> vollständige Vue-App
    // =========================================================

    let window_for_full =
        notesody_window.clone();

    let launcher_for_full =
        launcher.clone();

    let menu_open_for_full =
        menu_open.clone();

    let revealer_for_full =
        revealer.clone();


    full_button.connect_clicked(
        move |_| {

            menu_open_for_full.set(
                false
            );

            revealer_for_full
                .set_reveal_child(
                    false
                );

            // Das blaue Notizbuch + Menü verschwinden.
            launcher_for_full.hide();

            // Direkt die VOLLSTÄNDIGE App öffnen.
            open_full_window(
                &window_for_full
            );
        },
    );


    // Aufgabe:
    // vorerst Vollversion öffnen.
    let window_for_task =
        notesody_window.clone();

    let launcher_for_task =
        launcher.clone();

    task_button.connect_clicked(
        move |_| {

            launcher_for_task.hide();

            open_full_window(
                &window_for_task
            );
        },
    );


    // Prozess:
    // vorerst Vollversion öffnen.
    let window_for_process =
        notesody_window.clone();

    let launcher_for_process =
        launcher.clone();

    process_button.connect_clicked(
        move |_| {

            launcher_for_process.hide();

            open_full_window(
                &window_for_process
            );
        },
    );


    // =========================================================
    // BEFEHLE AUS VUE
    //
    // Wenn in der Vollversion auf das kleine Minus neben AB
    // geklickt wird:
    //
    // - Vue/Tauri-Fenster wird dort versteckt
    // - show_launcher wird aufgerufen
    // - hier erscheint wieder NUR das blaue Notizbuch
    // =========================================================

    let launcher_for_commands =
        launcher.clone();

    let revealer_for_commands =
        revealer.clone();

    let menu_open_for_commands =
        menu_open.clone();


    glib::timeout_add_local(
        Duration::from_millis(40),
        move || {

            while let Ok(command) =
                receiver.try_recv()
            {
                match command {

                    LauncherCommand::ShowCollapsed => {

                        menu_open_for_commands
                            .set(false);

                        revealer_for_commands
                            .set_reveal_child(false);

                        launcher_for_commands
                            .resize(
                                92,
                                104,
                            );

                        launcher_for_commands
                            .show_all();

                        launcher_for_commands
                            .present();
                    }
                }
            }

            glib::ControlFlow::Continue
        },
    );


    // =========================================================
    // START
    // =========================================================

    launcher.show_all();

    revealer.set_reveal_child(
        false
    );

    launcher.resize(
        92,
        104,
    );


    Ok(())
}
