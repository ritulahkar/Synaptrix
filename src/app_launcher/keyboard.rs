use gtk::{gdk, glib};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Entry, ListBox, ListBoxRow, EventControllerKey, Button};
use gtk::Box as GtkBox;

pub fn setup_keyboard_navigation(window: &ApplicationWindow, search_entry: &Entry, app_list: &ListBox) {
    let app_list_clone = app_list.clone();
    let search_entry_clone = search_entry.clone();
    let window_clone = window.clone();

    let key_controller = EventControllerKey::new();

    key_controller.connect_key_pressed(move |_, key, _, _| match key {
        gdk::Key::Down => {
            if let Some(selected) = app_list_clone.selected_row() {
                if let Some(next) = selected.next_sibling() {
                    if let Some(next_row) = next.downcast_ref::<ListBoxRow>() {
                        app_list_clone.select_row(Some(next_row));
                        next_row.grab_focus();
                    }
                }
            } else if let Some(first) = app_list_clone.row_at_index(0) {
                app_list_clone.select_row(Some(&first));
                first.grab_focus();
            }
            glib::Propagation::Stop
        }
        gdk::Key::Up => {
            if let Some(selected) = app_list_clone.selected_row() {
                if let Some(prev) = selected.prev_sibling() {
                    if let Some(prev_row) = prev.downcast_ref::<ListBoxRow>() {
                        app_list_clone.select_row(Some(prev_row));
                        prev_row.grab_focus();
                    }
                }
            }
            glib::Propagation::Stop
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
            if let Some(selected) = app_list_clone.selected_row() {
               // println!("DEBUG: Enter pressed on selected row");
                activate_selected_row(&selected);
            } else {
                // If no row is selected but there are items, select and activate the first one
                if let Some(first_row) = app_list_clone.row_at_index(0) {
                   // println!("DEBUG: Enter pressed with no selection, activating first row");
                    app_list_clone.select_row(Some(&first_row));
                    activate_selected_row(&first_row);
                }
            }
            glib::Propagation::Stop
        }
        gdk::Key::Escape => {
            window_clone.close();
            glib::Propagation::Stop
        }
        _ => {
            if !search_entry_clone.has_focus() {
                search_entry_clone.grab_focus();
            }
            glib::Propagation::Proceed
        }
    });

    window.add_controller(key_controller);

    // Handle Enter key in search entry
    let app_list_for_activate = app_list.clone();
    search_entry.connect_activate(move |_| {
       // println!("DEBUG: Enter pressed in search entry");
        if let Some(selected) = app_list_for_activate.selected_row() {
           // println!("DEBUG: Activating selected row from search entry");
            activate_selected_row(&selected);
        } else if let Some(first_row) = app_list_for_activate.row_at_index(0) {
           // println!("DEBUG: No selection, activating first row from search entry");
            app_list_for_activate.select_row(Some(&first_row));
            activate_selected_row(&first_row);
        }
    });
}

fn activate_selected_row(row: &ListBoxRow) {
   // println!("DEBUG: Attempting to activate row");
    if let Some(child) = row.child() {
       // println!("DEBUG: Row has child widget, searching for button");
        find_and_click_button(&child);
    } else {
       // println!("DEBUG: Row has no child widget");
    }
}

pub fn find_and_click_button(widget: &gtk::Widget) {
   // println!("DEBUG: Searching for button in widget: {}", widget.type_().name());
    
    // Check if this widget is a button
    if let Some(button) = widget.downcast_ref::<Button>() {
       // println!("DEBUG: Found button, clicking it");
        button.emit_clicked();
        return;
    }
    
    // Check if this is a container and search its children
    if let Some(container) = widget.downcast_ref::<GtkBox>() {
       // println!("DEBUG: Widget is a GtkBox, searching children");
        let mut child = container.first_child();
        while let Some(current_child) = child {
            find_and_click_button(&current_child);
            child = current_child.next_sibling();
        }
    } else if let Some(container) = widget.downcast_ref::<gtk::ListBoxRow>() {
       // println!("DEBUG: Widget is a ListBoxRow, searching child");
        if let Some(row_child) = container.child() {
            find_and_click_button(&row_child);
        }
    } else {
        // For other container types, try to iterate through children if possible
       // println!("DEBUG: Widget is not a known container type, trying generic child iteration");
        let mut child = widget.first_child();
        while let Some(current_child) = child {
            find_and_click_button(&current_child);
            child = current_child.next_sibling();
        }
    }
}