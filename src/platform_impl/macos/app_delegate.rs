use super::{activation_hack, app_state::AppState};
use cocoa::base::{id, nil, selector};
use cocoa::{
    appkit::{
        NSApplication, NSApplicationActivateIgnoringOtherApps, NSApplicationActivationPolicy,
        NSMenu, NSMenuItem, NSRunningApplication,
    },
    foundation::{NSAutoreleasePool, NSProcessInfo, NSString},
};
use objc::{
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
};
use std::os::raw::c_void;

pub struct AppDelegateClass(pub *const Class);
unsafe impl Send for AppDelegateClass {}
unsafe impl Sync for AppDelegateClass {}

lazy_static! {
    pub static ref APP_DELEGATE_CLASS: AppDelegateClass = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("WinitAppDelegate", superclass).unwrap();

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidBecomeActive:),
            did_become_active as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidResignActive:),
            did_resign_active as extern "C" fn(&Object, Sel, id),
        );

        decl.add_ivar::<*mut c_void>(activation_hack::State::name());
        decl.add_method(
            sel!(activationHackMouseMoved:),
            activation_hack::mouse_moved as extern "C" fn(&Object, Sel, id),
        );

        AppDelegateClass(decl.register())
    };
}

extern "C" fn new(class: &Class, _: Sel) -> id {
    unsafe {
        let this: id = msg_send![class, alloc];
        let this: id = msg_send![this, init];
        (*this).set_ivar(
            activation_hack::State::name(),
            activation_hack::State::new(),
        );
        this
    }
}

extern "C" fn dealloc(this: &Object, _: Sel) {
    unsafe {
        activation_hack::State::free(activation_hack::State::get_ptr(this));
    }
}

extern "C" fn did_finish_launching(_: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidFinishLaunching`");
    AppState::launched();

    unsafe {
        let ns_app = NSApplication::sharedApplication(nil);

        // Create menu bar
        let menubar = NSMenu::new(nil).autorelease();
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        menubar.addItem_(app_menu_item);
        ns_app.setMainMenu_(menubar);

        // Create Application menu
        let app_menu = NSMenu::new(nil).autorelease();
        let quit_prefix = NSString::alloc(nil).init_str("Quit ");
        let quit_title =
            quit_prefix.stringByAppendingString_(NSProcessInfo::processInfo(nil).processName());
        let quit_action = selector("terminate:");
        let quit_key = NSString::alloc(nil).init_str("q");
        let quit_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key)
            .autorelease();
        app_menu.addItem_(quit_item);
        app_menu_item.setSubmenu_(app_menu);
    }

    use self::NSApplicationActivationPolicy::*;

    unsafe {
        let ns_app = NSApplication::sharedApplication(nil);
        ns_app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
    }
    trace!("Completed `applicationDidFinishLaunching`");
}

extern "C" fn did_become_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidBecomeActive`");
    unsafe {
        activation_hack::State::set_activated(this, true);
    }
    trace!("Completed `applicationDidBecomeActive`");
}

extern "C" fn did_resign_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidResignActive`");
    unsafe {
        activation_hack::refocus(this);
    }
    trace!("Completed `applicationDidResignActive`");
}
