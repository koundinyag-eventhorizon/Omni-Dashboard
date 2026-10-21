#[cfg(target_os = "windows")]
mod windows_impl {
    use std::thread;
    use std::time::Duration;
    use uiautomation::UIAutomation;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    pub fn start_loop() {
        println!("👀 Background accessibility capture loop starting (Windows)...");

        let automation = UIAutomation::new().expect("Failed to init UI Automation");
        let walker = automation
            .get_control_view_walker()
            .expect("Failed to get tree walker");

        loop {
            thread::sleep(Duration::from_secs(5));

            unsafe {
                let hwnd = GetForegroundWindow();
                if hwnd.0 == 0 {
                    continue;
                }

                if let Ok(app_element) = automation.element_from_handle((hwnd.0 as isize).into()) {
                    let app_name = app_element
                        .get_name()
                        .unwrap_or_else(|_| "Unknown App".to_string());

                    let mut captured_text = String::new();
                    let _ = extract_text_from_element(&walker, &app_element, &mut captured_text, 0);

                    let cleaned_text = captured_text.trim();
                    if !cleaned_text.is_empty() {
                        println!("\n--- 🟢 ACTIVE APP: {} ---", app_name);

                        let display_text = if cleaned_text.len() > 300 {
                            format!(
                                "{}... [{} chars captured]",
                                &cleaned_text[..300],
                                cleaned_text.len()
                            )
                        } else {
                            cleaned_text.to_string()
                        };
                        println!("TEXT: {}", display_text);

                        if let Err(e) = crate::database::db::insert_capture(&app_name, cleaned_text) {
                            eprintln!("Failed to save memory to SQLite: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn extract_text_from_element(
        walker: &uiautomation::UITreeWalker,
        element: &uiautomation::UIElement,
        buffer: &mut String,
        depth: usize,
    ) -> uiautomation::Result<()> {
        if depth > 50 {
            return Ok(());
        }

        if let Ok(name) = element.get_name() {
            if !name.trim().is_empty() {
                buffer.push_str(&name);
                buffer.push_str(" | ");
            }
        }

        if let Ok(child) = walker.get_first_child(element) {
            let _ = extract_text_from_element(walker, &child, buffer, depth + 1);
            let mut next = child;
            while let Ok(sibling) = walker.get_next_sibling(&next) {
                let _ = extract_text_from_element(walker, &sibling, buffer, depth + 1);
                next = sibling;
            }
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod linux_impl {
    use std::thread;
    use std::time::Duration;
    use active_win_pos_rs::get_active_window;

    pub fn start_loop() {
        println!("👀 Background accessibility capture loop starting (Linux)...");

        loop {
            thread::sleep(Duration::from_secs(5));

            match get_active_window() {
                Ok(window) => {
                    let app_name = if window.app_name.is_empty() {
                        "Unknown App".to_string()
                    } else {
                        window.app_name.clone()
                    };

                    let captured_text = window.title.clone();
                    let cleaned_text = captured_text.trim();

                    if !cleaned_text.is_empty() {
                        println!("\n--- 🟢 ACTIVE APP: {} ---", app_name);

                        let display_text = if cleaned_text.len() > 300 {
                            format!(
                                "{}... [{} chars captured]",
                                &cleaned_text[..300],
                                cleaned_text.len()
                            )
                        } else {
                            cleaned_text.to_string()
                        };
                        println!("TEXT: {}", display_text);

                        if let Err(e) = crate::database::db::insert_capture(&app_name, cleaned_text) {
                            eprintln!("Failed to save memory to SQLite: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Could not get active window: {:?}", e);
                }
            }
        }
    }
}

pub fn start_loop() {
    #[cfg(target_os = "windows")]
    windows_impl::start_loop();

    #[cfg(not(target_os = "windows"))]
    linux_impl::start_loop();
}
