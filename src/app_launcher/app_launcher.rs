// app_launcher.rs - Complete implementation with X11 centering (compatibility fixed)
use super::{desktop_parser, file_loader, keyboard, search, styles, ui};
use crate::app_info::AppInfo;
use crate::settings::LauncherSettings;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, ListBox};
use std::collections::HashMap;

// X11 imports - add these dependencies to Cargo.toml:
// [dependencies]
// x11 = "2.21"
// libc = "0.2"
use x11::xlib::*;
use std::ptr;

// Xinerama structure definition
#[repr(C)]
#[derive(Clone, Copy)]
struct XineramaScreenInfo {
    screen_number: i32,
    x_org: i32,
    y_org: i32,
    width: i32,
    height: i32,
}

// External function declarations for Xinerama
extern "C" {
    fn XineramaQueryScreens(display: *mut Display, number: *mut i32) -> *mut XineramaScreenInfo;
    fn XineramaIsActive(display: *mut Display) -> i32;
}

pub struct X11WindowCentering {
    display: *mut Display,
}

impl X11WindowCentering {
    pub fn new() -> Option<Self> {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                eprintln!("Failed to open X11 display");
                return None;
            }
            Some(Self { display })
        }
    }

    // Helper method to get X11 window ID from GTK window using title matching
    fn get_x11_window_id(&self, gtk_window: &gtk::ApplicationWindow) -> Result<Window, String> {
        let window_title = gtk_window.title().unwrap_or_else(|| "Launcher".into());
        
        unsafe {
            let root = XDefaultRootWindow(self.display);
            self.find_window_by_title(root, &window_title)
                .ok_or_else(|| format!("Could not find X11 window with title '{}'", window_title))
        }
    }

    // Recursively search for window by title
    unsafe fn find_window_by_title(&self, window: Window, target_title: &str) -> Option<Window> {
        // Check if current window matches
        if let Some(title) = self.get_window_title(window) {
            if title == target_title {
                return Some(window);
            }
        }

        // Search children
        let mut root_return = 0;
        let mut parent_return = 0;
        let mut children_return = ptr::null_mut();
        let mut nchildren_return = 0;

        if XQueryTree(
            self.display,
            window,
            &mut root_return,
            &mut parent_return,
            &mut children_return,
            &mut nchildren_return,
        ) != 0 && !children_return.is_null()
        {
            let children = std::slice::from_raw_parts(children_return, nchildren_return as usize);
            for &child in children {
                if let Some(found) = self.find_window_by_title(child, target_title) {
                    XFree(children_return as *mut _);
                    return Some(found);
                }
            }
            XFree(children_return as *mut _);
        }

        None
    }

    unsafe fn get_window_title(&self, window: Window) -> Option<String> {
        let mut name = ptr::null_mut();
        if XFetchName(self.display, window, &mut name) != 0 && !name.is_null() {
            let title = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
            XFree(name as *mut _);
            Some(title)
        } else {
            None
        }
    }

    pub fn center_window(&self, gtk_window: &gtk::ApplicationWindow) -> Result<(), String> {
        // Alternative approach: use window class name instead of title
        let xid = self.find_gtk_window(gtk_window)?;

        unsafe {
            // Get screen dimensions
            let screen = XDefaultScreen(self.display);
            let screen_width = XDisplayWidth(self.display, screen);
            let screen_height = XDisplayHeight(self.display, screen);

            // Get window attributes to find current size
            let mut window_attrs = std::mem::zeroed::<XWindowAttributes>();
            if XGetWindowAttributes(self.display, xid, &mut window_attrs) == 0 {
                return Err("Failed to get window attributes".to_string());
            }

            // Calculate center position
            let window_width = window_attrs.width;
            let window_height = window_attrs.height;
            let x = (screen_width - window_width) / 2;
            let y = (screen_height - window_height) / 2;

            // Move window to center
            XMoveWindow(self.display, xid, x, y);
            XFlush(self.display);

            println!("Centered window at ({}, {}) on screen {}x{}", x, y, screen_width, screen_height);
        }

        Ok(())
    }

    // Find GTK window by looking for specific window properties
    fn find_gtk_window(&self, _gtk_window: &gtk::ApplicationWindow) -> Result<Window, String> {
        unsafe {
            let root = XDefaultRootWindow(self.display);
            
            // Look for windows with specific properties that identify our GTK app
            if let Some(window) = self.find_window_by_class(root, "synaptrix") {
                return Ok(window);
            }
            
            // Fallback: look for any recently created window
            if let Some(window) = self.find_most_recent_window(root) {
                return Ok(window);
            }

            Err("Could not find GTK window".to_string())
        }
    }

    unsafe fn find_window_by_class(&self, window: Window, target_class: &str) -> Option<Window> {
        // Check current window's class
        let mut class_hints = std::mem::zeroed::<XClassHint>();
        if XGetClassHint(self.display, window, &mut class_hints) != 0 {
            let mut matches = false;
            
            if !class_hints.res_class.is_null() {
                let class_name = std::ffi::CStr::from_ptr(class_hints.res_class)
                    .to_string_lossy()
                    .to_lowercase();
                matches = class_name.contains(&target_class.to_lowercase());
                XFree(class_hints.res_class as *mut _);
            }
            
            if !class_hints.res_name.is_null() {
                let res_name = std::ffi::CStr::from_ptr(class_hints.res_name)
                    .to_string_lossy()
                    .to_lowercase();
                matches = matches || res_name.contains(&target_class.to_lowercase());
                XFree(class_hints.res_name as *mut _);
            }

            if matches {
                return Some(window);
            }
        }

        // Search children
        let mut root_return = 0;
        let mut parent_return = 0;
        let mut children_return = ptr::null_mut();
        let mut nchildren_return = 0;

        if XQueryTree(
            self.display,
            window,
            &mut root_return,
            &mut parent_return,
            &mut children_return,
            &mut nchildren_return,
        ) != 0 && !children_return.is_null()
        {
            let children = std::slice::from_raw_parts(children_return, nchildren_return as usize);
            for &child in children {
                if let Some(found) = self.find_window_by_class(child, target_class) {
                    XFree(children_return as *mut _);
                    return Some(found);
                }
            }
            XFree(children_return as *mut _);
        }

        None
    }

    unsafe fn find_most_recent_window(&self, root: Window) -> Option<Window> {
        let mut root_return = 0;
        let mut parent_return = 0;
        let mut children_return = ptr::null_mut();
        let mut nchildren_return = 0;

        if XQueryTree(
            self.display,
            root,
            &mut root_return,
            &mut parent_return,
            &mut children_return,
            &mut nchildren_return,
        ) != 0 && !children_return.is_null()
        {
            let children = std::slice::from_raw_parts(children_return, nchildren_return as usize);
            
            // Return the last child window (most recently created)
            if let Some(&last_window) = children.last() {
                // Check if it's a visible window
                let mut attrs = std::mem::zeroed::<XWindowAttributes>();
                if XGetWindowAttributes(self.display, last_window, &mut attrs) != 0 
                    && attrs.map_state == IsViewable 
                {
                    XFree(children_return as *mut _);
                    return Some(last_window);
                }
            }
            
            XFree(children_return as *mut _);
        }

        None
    }

    pub fn center_window_on_primary_monitor(&self, gtk_window: &gtk::ApplicationWindow) -> Result<(), String> {
        let xid = self.find_gtk_window(gtk_window)?;

        unsafe {
            // Check if Xinerama is active
            if XineramaIsActive(self.display) == 0 {
                // Fallback to default screen centering
                return self.center_window(gtk_window);
            }

            // Get monitor information using Xinerama
            let mut num_monitors = 0;
            let monitors = XineramaQueryScreens(self.display, &mut num_monitors);
            
            if monitors.is_null() || num_monitors == 0 {
                // Fallback to default screen centering
                return self.center_window(gtk_window);
            }

            // Use the first monitor (usually primary)
            let monitor = *monitors;
            
            // Get window size
            let mut window_attrs = std::mem::zeroed::<XWindowAttributes>();
            if XGetWindowAttributes(self.display, xid, &mut window_attrs) == 0 {
                XFree(monitors as *mut _);
                return Err("Failed to get window attributes".to_string());
            }

            // Calculate center position on primary monitor
            let x = monitor.x_org + (monitor.width - window_attrs.width) / 2;
            let y = monitor.y_org + (monitor.height - window_attrs.height) / 2;

            XMoveWindow(self.display, xid, x, y);
            XFlush(self.display);
            XFree(monitors as *mut _);

            println!("Centered window on primary monitor at ({}, {})", x, y);
        }

        Ok(())
    }

    pub fn center_window_on_monitor(&self, gtk_window: &gtk::ApplicationWindow, monitor_index: i32) -> Result<(), String> {
        let xid = self.find_gtk_window(gtk_window)?;

        unsafe {
            // Check if Xinerama is active
            if XineramaIsActive(self.display) == 0 {
                return self.center_window(gtk_window);
            }

            // Get monitor information using Xinerama
            let mut num_monitors = 0;
            let monitors = XineramaQueryScreens(self.display, &mut num_monitors);
            
            if monitors.is_null() || monitor_index >= num_monitors || monitor_index < 0 {
                // Fallback to primary monitor or default screen
                XFree(monitors as *mut _);
                return if num_monitors > 0 {
                    self.center_window_on_primary_monitor(gtk_window)
                } else {
                    self.center_window(gtk_window)
                };
            }

            let monitor = *monitors.offset(monitor_index as isize);
            
            // Get window size
            let mut window_attrs = std::mem::zeroed::<XWindowAttributes>();
            if XGetWindowAttributes(self.display, xid, &mut window_attrs) == 0 {
                XFree(monitors as *mut _);
                return Err("Failed to get window attributes".to_string());
            }

            // Calculate center position on specific monitor
            let x = monitor.x_org + (monitor.width - window_attrs.width) / 2;
            let y = monitor.y_org + (monitor.height - window_attrs.height) / 2;

            XMoveWindow(self.display, xid, x, y);
            XFlush(self.display);
            XFree(monitors as *mut _);

            println!("Centered window on monitor {} at ({}, {})", monitor_index, x, y);
        }

        Ok(())
    }

    pub fn get_monitor_info(&self) -> Vec<(i32, i32, i32, i32)> {
        let mut monitors = Vec::new();
        
        unsafe {
            if XineramaIsActive(self.display) == 0 {
                // Single screen setup
                let screen = XDefaultScreen(self.display);
                let width = XDisplayWidth(self.display, screen);
                let height = XDisplayHeight(self.display, screen);
                monitors.push((0, 0, width, height));
                return monitors;
            }

            let mut num_monitors = 0;
            let monitor_array = XineramaQueryScreens(self.display, &mut num_monitors);
            
            if !monitor_array.is_null() {
                for i in 0..num_monitors {
                    let monitor = *monitor_array.offset(i as isize);
                    monitors.push((monitor.x_org, monitor.y_org, monitor.width, monitor.height));
                }
                XFree(monitor_array as *mut _);
            }
        }
        
        monitors
    }
}

impl Drop for X11WindowCentering {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XCloseDisplay(self.display);
            }
        }
    }
}

pub struct AppLauncher {
    apps: HashMap<String, AppInfo>,
    recent_files: Vec<AppInfo>,
    window: ApplicationWindow,
    search_entry: Entry,
    app_list: ListBox,
    settings: LauncherSettings,
    app_ref: Option<Application>, // Store reference to quit/hide the app
}

impl AppLauncher {
    pub fn new(app: &Application) -> Self {
        let settings = LauncherSettings::load();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Launcher")
            .default_width(settings.window.width)
            .default_height(settings.window.height)
            .decorated(false)
            .resizable(false)
            .build();

        // Make window modal
        window.set_modal(true);
        window.set_icon_name(Some("synaptrix"));

        let mut launcher = Self {
            apps: HashMap::new(),
            recent_files: Vec::new(),
            window,
            search_entry: Entry::new(),
            app_list: ListBox::new(),
            settings,
            app_ref: Some(app.clone()),
        };

        launcher.setup();
        launcher
    }

    fn setup(&mut self) {
        ui::setup_ui(
            &self.window,
            &mut self.search_entry,
            &mut self.app_list,
            &self.settings,
        );
        styles::setup_styles(&self.window, &self.settings);

        // Setup close request handler AFTER UI is set up
        self.setup_close_handler();

        // Modified focus out handler to hide instead of close
        self.setup_hide_on_focus_out();

        desktop_parser::load_applications(&mut self.apps);
        file_loader::load_recent_files(&mut self.recent_files, &self.settings);
        keyboard::setup_keyboard_navigation(&self.window, &self.search_entry, &self.app_list);
        search::setup_search(
            &self.search_entry,
            &self.app_list,
            &self.apps,
            &self.recent_files,
            &self.settings,
        );

        // Setup enhanced keyboard handlers
        self.setup_keyboard_handlers();

        self.populate_list("");
    }

    fn setup_close_handler(&self) {
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;

        if quit_on_close {
            println!("Setting up NORMAL mode close handler");
            self.window.connect_close_request(move |_window| {
                println!("Close request received in NORMAL mode");
                if let Some(app) = app_weak.upgrade() {
                    println!("Calling app.quit()");
                    app.quit();
                } else {
                    println!("No application found!");
                }
                gtk::glib::Propagation::Stop
            });
        } else {
            println!("Setting up DAEMON mode close handler");
            self.window.connect_close_request(|window| {
                println!("Close request received in DAEMON mode - hiding window");
                window.set_visible(false);
                gtk::glib::Propagation::Stop
            });
        }
    }

    fn setup_hide_on_focus_out(&self) {
        let window_weak = self.window.downgrade();
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;
        let focus_controller = gtk::EventControllerFocus::new();

        focus_controller.connect_leave(move |_| {
            let app_weak = app_weak.clone();
            if let Some(window) = window_weak.upgrade() {
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(100),
                    move || {
                        if window.has_focus() {
                            return;
                        }

                        if let Some(focus_widget) = window.focus_widget() {
                            if focus_widget.has_focus() {
                                return;
                            }
                        }

                        // Respect the quit_on_close setting
                        if quit_on_close {
                            // println!("Focus lost in normal mode - quitting app");
                            if let Some(app) = app_weak.upgrade() {
                                app.quit();
                            }
                        } else {
                            // println!("Focus lost in daemon mode - hiding window");
                            window.set_visible(false);
                        }
                    },
                );
            }
        });

        self.window.add_controller(focus_controller);
    }

    fn setup_keyboard_handlers(&self) {
        let window_weak = self.window.downgrade();
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;
        let key_controller = gtk::EventControllerKey::new();

        key_controller.connect_key_pressed(move |_, key, _, modifier| {
            match key {
                gtk::gdk::Key::Escape => {
                    if let Some(window) = window_weak.upgrade() {
                        if quit_on_close {
                            // Quit the entire application
                            if let Some(app) = app_weak.upgrade() {
                                println!("ESC pressed - quitting app in normal mode");
                                app.quit();
                            }
                        } else {
                            // Just hide the window (daemon mode)
                            println!("ESC pressed - hiding window in daemon mode");
                            window.set_visible(false);
                        }
                        return gtk::glib::Propagation::Stop;
                    }
                }
                gtk::gdk::Key::q | gtk::gdk::Key::Q => {
                    // Ctrl+Q to always quit the entire application regardless of settings
                    if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                        if let Some(app) = app_weak.upgrade() {
                            println!("Ctrl+Q pressed - quitting app");
                            app.quit();
                            return gtk::glib::Propagation::Stop;
                        }
                    }
                }
                _ => {}
            }
            gtk::glib::Propagation::Proceed
        });

        self.window.add_controller(key_controller);
    }

    fn populate_list(&mut self, query: &str) {
        search::filter_and_populate(
            &self.app_list,
            &self.apps,
            &self.recent_files,
            query,
            &self.settings,
        );
    }

    pub fn show(&self) {
        // Clear search when showing
        self.search_entry.set_text("");

        // Re-populate the list with all items
        search::filter_and_populate(
            &self.app_list,
            &self.apps,
            &self.recent_files,
            "",
            &self.settings,
        );

        // Show the window first
        self.window.set_visible(true);
        self.window.present();

        // Center the window using X11 after it's visible
        self.center_window_delayed();

        self.search_entry.grab_focus();
    }

    fn center_window_delayed(&self) {
        if let Some(x11_centering) = X11WindowCentering::new() {
            // Wait a bit for the window to be fully realized
            let window_weak = self.window.downgrade();
            gtk::glib::timeout_add_local_once(
                std::time::Duration::from_millis(200), // Increased delay for better reliability
                move || {
                    if let Some(window) = window_weak.upgrade() {
                        // Try primary monitor centering first, fallback to basic centering
                        if let Err(e) = x11_centering.center_window_on_primary_monitor(&window) {
                            eprintln!("Failed to center on primary monitor: {}, trying basic centering", e);
                            if let Err(e2) = x11_centering.center_window(&window) {
                                eprintln!("Failed to center window: {}", e2);
                            }
                        }
                    }
                }
            );
        } else {
            eprintln!("Failed to initialize X11 window centering - not running on X11?");
        }
    }

    // Alternative method for immediate centering (call after window is mapped)
    pub fn center_window_now(&self) -> Result<(), String> {
        let x11_centering = X11WindowCentering::new()
            .ok_or("Failed to initialize X11 connection")?;
        x11_centering.center_window_on_primary_monitor(&self.window)
    }

    // Center on specific monitor
    pub fn center_window_on_monitor(&self, monitor_index: i32) -> Result<(), String> {
        let x11_centering = X11WindowCentering::new()
            .ok_or("Failed to initialize X11 connection")?;
        x11_centering.center_window_on_monitor(&self.window, monitor_index)
    }

    // Get available monitors
    pub fn get_monitors(&self) -> Vec<(i32, i32, i32, i32)> {
        if let Some(x11_centering) = X11WindowCentering::new() {
            x11_centering.get_monitor_info()
        } else {
            Vec::new()
        }
    }

    // Getters for accessing components
    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    pub fn search_entry(&self) -> &Entry {
        &self.search_entry
    }

    pub fn app_list(&self) -> &ListBox {
        &self.app_list
    }

    pub fn settings(&self) -> &LauncherSettings {
        &self.settings
    }
}

// Optional: Helper function to test X11 functionality
pub fn test_x11_centering() {
    if let Some(x11) = X11WindowCentering::new() {
        let monitors = x11.get_monitor_info();
        println!("Available monitors:");
        for (i, (x, y, w, h)) in monitors.iter().enumerate() {
            println!("  Monitor {}: {}x{} at ({}, {})", i, w, h, x, y);
        }
    } else {
        println!("X11 not available");
    }
}