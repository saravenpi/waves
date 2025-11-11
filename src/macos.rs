use std::sync::Mutex;
use std::path::PathBuf;

pub static FILE_PATH_CHANNEL: Mutex<Option<std::sync::mpsc::Sender<PathBuf>>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    OpenFile,
    OpenFolder,
}

pub static MENU_ACTION_CHANNEL: Mutex<Option<std::sync::mpsc::Sender<MenuAction>>> = Mutex::new(None);

#[cfg(target_os = "macos")]
pub fn setup_file_open_handler(sender: std::sync::mpsc::Sender<PathBuf>) {
    use cocoa::base::{id, nil, YES};
    use cocoa::appkit::NSApplication;
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};

    *FILE_PATH_CHANNEL.lock().unwrap() = Some(sender);

    unsafe {
        let app: id = NSApplication::sharedApplication(nil);

        // Create our delegate class that implements NSApplicationDelegate
        let delegate_class = if let Some(mut decl) = ClassDecl::new("WavesAppDelegate", class!(NSObject)) {
            // Implement application:openFile: for single file
            extern "C" fn application_open_file(
                _this: &Object,
                _cmd: Sel,
                _app: id,
                filename: id,
            ) -> bool {
                unsafe {
                    eprintln!("WAVES: application:openFile: called");

                    let cstring: *const i8 = msg_send![filename, UTF8String];
                    if !cstring.is_null() {
                        let path_str = std::ffi::CStr::from_ptr(cstring)
                            .to_str()
                            .unwrap_or("")
                            .to_string();

                        eprintln!("WAVES: Opening file from delegate: {}", path_str);

                        if let Ok(guard) = FILE_PATH_CHANNEL.lock() {
                            if let Some(sender) = &*guard {
                                let _ = sender.send(PathBuf::from(path_str));
                                return true;
                            }
                        }
                    }
                    false
                }
            }

            // Implement application:openFiles: for multiple files
            extern "C" fn application_open_files(
                _this: &Object,
                _cmd: Sel,
                _app: id,
                filenames: id,
            ) {
                unsafe {
                    eprintln!("WAVES: application:openFiles: called");

                    let count: usize = msg_send![filenames, count];
                    eprintln!("WAVES: Processing {} files", count);

                    for i in 0..count {
                        let filename: id = msg_send![filenames, objectAtIndex: i];
                        let cstring: *const i8 = msg_send![filename, UTF8String];
                        if !cstring.is_null() {
                            let path_str = std::ffi::CStr::from_ptr(cstring)
                                .to_str()
                                .unwrap_or("")
                                .to_string();

                            eprintln!("WAVES: Opening file from delegate: {}", path_str);

                            if let Ok(guard) = FILE_PATH_CHANNEL.lock() {
                                if let Some(sender) = &*guard {
                                    let _ = sender.send(PathBuf::from(path_str));
                                }
                            }
                        }
                    }
                }
            }

            decl.add_method(
                sel!(application:openFile:),
                application_open_file as extern "C" fn(&Object, Sel, id, id) -> bool,
            );

            decl.add_method(
                sel!(application:openFiles:),
                application_open_files as extern "C" fn(&Object, Sel, id, id),
            );

            decl.register()
        } else {
            Class::get("WavesAppDelegate").expect("Failed to get WavesAppDelegate class")
        };

        let app_delegate: id = msg_send![delegate_class, new];

        // Set as the application delegate
        let _: () = msg_send![app, setDelegate: app_delegate];

        eprintln!("WAVES: Application delegate set");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn setup_file_open_handler(_sender: std::sync::mpsc::Sender<PathBuf>) {
    // No-op on non-macOS platforms
}

#[cfg(target_os = "macos")]
pub fn setup_menu_bar(menu_action_sender: std::sync::mpsc::Sender<MenuAction>) {
    use cocoa::appkit::{NSApp, NSApplication, NSMenu, NSMenuItem};
    use cocoa::base::{id, nil, selector};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};

    *MENU_ACTION_CHANNEL.lock().unwrap() = Some(menu_action_sender);

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        // Create menu delegate class
        let delegate_class = if let Some(mut decl) = ClassDecl::new("WavesMenuDelegate", class!(NSObject)) {
            extern "C" fn open_file(_this: &Object, _cmd: Sel, _sender: id) {
                if let Ok(guard) = MENU_ACTION_CHANNEL.lock() {
                    if let Some(sender) = &*guard {
                        let _ = sender.send(MenuAction::OpenFile);
                    }
                }
            }

            extern "C" fn open_folder(_this: &Object, _cmd: Sel, _sender: id) {
                if let Ok(guard) = MENU_ACTION_CHANNEL.lock() {
                    if let Some(sender) = &*guard {
                        let _ = sender.send(MenuAction::OpenFolder);
                    }
                }
            }

            decl.add_method(
                sel!(openFile:),
                open_file as extern "C" fn(&Object, Sel, id),
            );

            decl.add_method(
                sel!(openFolder:),
                open_folder as extern "C" fn(&Object, Sel, id),
            );

            decl.register()
        } else {
            Class::get("WavesMenuDelegate").expect("Failed to get WavesMenuDelegate class")
        };

        let delegate: id = msg_send![delegate_class, new];

        // Get the application
        let app = NSApp();

        // Create main menu bar
        let main_menu = NSMenu::new(nil);
        let _: () = msg_send![main_menu, setAutoenablesItems: false];

        // Create app menu item (leftmost menu)
        let app_menu_item = NSMenuItem::new(nil);
        let _: () = msg_send![main_menu, addItem: app_menu_item];

        let app_menu = NSMenu::new(nil);
        let _: () = msg_send![app_menu_item, setSubmenu: app_menu];

        let quit_title = NSString::alloc(nil);
        let quit_title: id = msg_send![quit_title, initWithUTF8String: "Quit Waves\0".as_ptr() as *const i8];
        let quit_action = selector("terminate:");
        let quit_key = NSString::alloc(nil);
        let quit_key: id = msg_send![quit_key, initWithUTF8String: "q\0".as_ptr() as *const i8];
        let quit_item = NSMenuItem::alloc(nil);
        let quit_item: id = msg_send![quit_item,
            initWithTitle: quit_title
            action: quit_action
            keyEquivalent: quit_key
        ];
        let _: () = msg_send![app_menu, addItem: quit_item];

        // Create File menu
        let file_menu_item = NSMenuItem::new(nil);
        let _: () = msg_send![main_menu, addItem: file_menu_item];

        let file_menu = NSMenu::new(nil);
        let file_menu_title = NSString::alloc(nil);
        let file_menu_title: id = msg_send![file_menu_title, initWithUTF8String: "File\0".as_ptr() as *const i8];
        let _: () = msg_send![file_menu, setTitle: file_menu_title];
        let _: () = msg_send![file_menu_item, setSubmenu: file_menu];

        // Add "Open File..." menu item
        let open_file_title = NSString::alloc(nil);
        let open_file_title: id = msg_send![open_file_title, initWithUTF8String: "Open File...\0".as_ptr() as *const i8];
        let open_file_action = selector("openFile:");
        let open_file_key = NSString::alloc(nil);
        let open_file_key: id = msg_send![open_file_key, initWithUTF8String: "o\0".as_ptr() as *const i8];
        let open_file_item = NSMenuItem::alloc(nil);
        let open_file_item: id = msg_send![open_file_item,
            initWithTitle: open_file_title
            action: open_file_action
            keyEquivalent: open_file_key
        ];
        let _: () = msg_send![open_file_item, setTarget: delegate];
        let _: () = msg_send![file_menu, addItem: open_file_item];

        // Add "Open Folder..." menu item
        let open_folder_title = NSString::alloc(nil);
        let open_folder_title: id = msg_send![open_folder_title, initWithUTF8String: "Open Folder...\0".as_ptr() as *const i8];
        let open_folder_action = selector("openFolder:");
        let open_folder_key = NSString::alloc(nil);
        let open_folder_key: id = msg_send![open_folder_key, initWithUTF8String: "O\0".as_ptr() as *const i8];
        let open_folder_item = NSMenuItem::alloc(nil);
        let open_folder_item: id = msg_send![open_folder_item,
            initWithTitle: open_folder_title
            action: open_folder_action
            keyEquivalent: open_folder_key
        ];
        let _: () = msg_send![open_folder_item, setTarget: delegate];
        let _: () = msg_send![file_menu, addItem: open_folder_item];

        // Set the main menu
        let _: () = msg_send![app, setMainMenu: main_menu];

        eprintln!("WAVES: Menu bar configured");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn setup_menu_bar(_menu_action_sender: std::sync::mpsc::Sender<MenuAction>) {
    // No-op on non-macOS platforms
}
