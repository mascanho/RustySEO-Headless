//! Detects whether a native "Save As" dialog can actually be shown, so exports
//! can fall back to writing straight to disk when the app is running headless
//! (e.g. over SSH on a server with no display attached).

/// True when a native file dialog should be able to open on this session.
pub fn gui_available() -> bool {
    gui_available_from(
        cfg!(target_os = "windows"),
        cfg!(target_os = "macos"),
        std::env::var_os("SSH_TTY").is_some() || std::env::var_os("SSH_CONNECTION").is_some(),
        std::env::var_os("DISPLAY").is_some(),
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
    )
}

/// Pure decision logic, kept separate from env lookups so it can be unit tested
/// without mutating process-global environment variables.
fn gui_available_from(is_windows: bool, is_macos: bool, is_ssh: bool, has_x11: bool, has_wayland: bool) -> bool {
    if is_windows {
        // Headless Windows Server Core exists but is rare enough not to special-case.
        return true;
    }
    if is_macos {
        // A native NSSavePanel can't reach a remote terminal over plain SSH; X11
        // forwarding (rare on macOS, but possible) is the one case it still can.
        return !is_ssh || has_x11;
    }
    // Linux / other Unix: only an actual display server can show a window.
    has_x11 || has_wayland
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_always_has_a_gui() {
        assert!(gui_available_from(true, false, true, false, false));
    }

    #[test]
    fn macos_local_session_has_a_gui() {
        assert!(gui_available_from(false, true, false, false, false));
    }

    #[test]
    fn macos_over_plain_ssh_has_no_gui() {
        assert!(!gui_available_from(false, true, true, false, false));
    }

    #[test]
    fn macos_over_ssh_with_x11_forwarding_has_a_gui() {
        assert!(gui_available_from(false, true, true, true, false));
    }

    #[test]
    fn linux_with_x11_has_a_gui() {
        assert!(gui_available_from(false, false, false, true, false));
    }

    #[test]
    fn linux_with_wayland_has_a_gui() {
        assert!(gui_available_from(false, false, false, false, true));
    }

    #[test]
    fn headless_linux_server_has_no_gui() {
        assert!(!gui_available_from(false, false, false, false, false));
    }

    #[test]
    fn linux_over_ssh_without_forwarding_has_no_gui() {
        assert!(!gui_available_from(false, false, true, false, false));
    }
}
