use gtk::{gdk, ApplicationWindow};
use crate::settings::LauncherSettings;

pub fn setup_styles(_window: &ApplicationWindow, settings: &LauncherSettings) {
    let css_provider = gtk::CssProvider::new();
    let css_content = format!(
        r#"
        /* Main window with clean light mode design */
        window {{
            background: {};
            border-radius: 12px;
            border: 1px solid rgba(0, 0, 0, 0.1);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
        }}
        
        /* Clean search entry optimized for light mode */
        .search-entry {{
            background: #ffffff;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            color: #2c3e50;
            font-size: 16px;
            font-weight: 500;
            padding: 14px 16px;
            margin: 12px;
            transition: all 200ms ease;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
        }}
        
        .search-entry:focus {{
            border-color: {};
            background: #ffffff;
            color: #2c3e50;
            box-shadow: 
                0 0 0 3px rgba(52, 152, 219, 0.1),
                0 4px 12px rgba(0, 0, 0, 0.1);
            outline: none;
        }}
        
        /* Removed ::selection as it's not supported in GTK CSS */
        
        /* App list container */
        .app-list {{
            background: transparent;
            border: none;
        }}
        
        scrolledwindow {{
            background: transparent;
            border: none;
        }}
        
        /* Clean app row design */
        .app-row {{
            background: #ffffff;
            border-radius: 8px;
            margin: 3px 8px;
            padding: 12px 16px;
            border: 1px solid #e8e8e8;
            transition: all 150ms ease;
        }}
        
        .app-row:hover {{
            background: #f8f9fa;
            border-color: #d0d0d0;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.08);
        }}
        
        .app-row:selected {{
            background: {};
            border-color: {};
            color: #ffffff;
            box-shadow: 0 4px 12px rgba(52, 152, 219, 0.2);
        }}
        
        /* Clear, readable typography */
        .app-name {{
            color: {};
            font-weight: 600;
            font-size: 14px;
            margin-bottom: 2px;
        }}
        
        .app-row:selected .app-name {{
            color: #ffffff;
            font-weight: 600;
        }}
        
        .app-description {{
            color: #6c757d;
            font-size: 12px;
            font-weight: 400;
            line-height: 1.3;
        }}
        
        .app-row:selected .app-description {{
            color: rgba(255, 255, 255, 0.9);
        }}
        
        /* Clean launch button */
        .launch-button {{
            background: {};
            border: none;
            border-radius: 6px;
            color: #ffffff;
            font-weight: 600;
            font-size: 12px;
            padding: 8px 14px;
            transition: all 150ms ease;
        }}
        
        .launch-button:hover {{
            background: #2980b9;
        }}
        
        /* Command rows with subtle green accent */
        .command-row {{
            background: #ffffff;
            border-radius: 8px;
            margin: 3px 8px;
            padding: 12px 16px;
            border: 1px solid #d4edda;
            transition: all 150ms ease;
            border-left: 4px solid #28a745;
        }}
        
        .command-row:hover {{
            background: #f8fff9;
            border-color: #c3e6cb;
            box-shadow: 0 4px 8px rgba(40, 167, 69, 0.1);
        }}
        
        .command-row:selected {{
            background: #28a745;
            border-color: #28a745;
            color: #ffffff;
            box-shadow: 0 4px 12px rgba(40, 167, 69, 0.2);
        }}
        
        .command-row:selected .app-name {{
            color: #ffffff;
        }}
        
        .command-row:selected .app-description {{
            color: rgba(255, 255, 255, 0.9);
        }}
        
        /* File rows with subtle orange accent */
        .file-row {{
            background: #ffffff;
            border-radius: 8px;
            margin: 3px 8px;
            padding: 12px 16px;
            border: 1px solid #ffeaa7;
            transition: all 150ms ease;
            border-left: 4px solid #fd7e14;
        }}
        
        .file-row:hover {{
            background: #fffbf0;
            border-color: #ffd32a;
            box-shadow: 0 4px 8px rgba(253, 126, 20, 0.1);
        }}
        
        .file-row:selected {{
            background: #fd7e14;
            border-color: #fd7e14;
            color: #ffffff;
            box-shadow: 0 4px 12px rgba(253, 126, 20, 0.2);
        }}
        
        .file-row:selected .app-name {{
            color: #ffffff;
        }}
        
        .file-row:selected .app-description {{
            color: rgba(255, 255, 255, 0.9);
        }}
        
        /* Clean section separators */
        .section-separator {{
            background: #e9ecef;
            min-height: 1px;
            margin: 8px 16px;
        }}
        
        .section-start {{
            border-top: 1px solid #dee2e6;
            margin-top: 8px;
            padding-top: 8px;
        }}

        .command-row.section-start {{
            border-top-color: #c3e6cb;
        }}

        .file-row.section-start {{
            border-top-color: #ffeaa7;
        }}
        
        /* Clean scrollbar */
        scrollbar {{
            background: transparent;
            border: none;
            padding: 2px;
        }}
        
        scrollbar slider {{
            background: #ced4da;
            border-radius: 4px;
            min-width: 6px;
            min-height: 6px;
            border: none;
            transition: background 150ms ease;
        }}
        
        scrollbar slider:hover {{
            background: #adb5bd;
        }}
        
        scrollbar slider:active {{
            background: #6c757d;
        }}
        
        /* Ensure good contrast in all states */
        .app-row:focus,
        .command-row:focus,
        .file-row:focus {{
            outline: 2px solid {};
            outline-offset: 2px;
        }}
        "#,
        settings.theme.background_color,     // window background
        settings.theme.accent_color,        // search entry focus border
        settings.theme.accent_color,        // app-row selected background
        settings.theme.accent_color,        // app-row selected border
        settings.theme.text_color,          // app-name color
        settings.theme.accent_color,        // launch button background
        settings.theme.accent_color,        // focus outline
    );

    css_provider.load_from_data(&css_content);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}