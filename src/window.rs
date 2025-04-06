use std::sync::{Arc, Mutex};

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, EndPaint, GetDC, ReleaseDC, SetStretchBltMode, StretchDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HALFTONE, HDC, PAINTSTRUCT, RGBQUAD, SRCCOPY,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

use crate::Bitmap;
const WINDOW_CLASS_NAME: &str = "RustRTWindowClass";

#[derive(Default)]
pub struct WindowDim {
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
pub struct Win32BackBuffer {
    pub bitmap: Arc<Mutex<Bitmap>>,
    pub info: BITMAPINFO,
}

pub struct Window {
    pub handle: HWND,
    pub shutdown_requested: bool,
    pub buffer: Win32BackBuffer,
    pub dim: WindowDim,
}

impl Window {
    pub fn new(window_name: &str, width: i32, height: i32, bitmap: Bitmap) -> Box<Self> {
        unsafe {
            let instance = GetModuleHandleA(None).expect("Could no get module handle... PANIC!");
            let window_class = WNDCLASSA {
                hInstance: instance.into(),
                lpszClassName: PCSTR(WINDOW_CLASS_NAME.as_ptr()),
                style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
                lpfnWndProc: Some(Window::window_proc),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                ..Default::default()
            };

            let mut window = Box::new(Window {
                handle: HWND::default(),
                shutdown_requested: false,
                buffer: Win32BackBuffer::default(),
                dim: WindowDim::default(),
            });

            assert_ne!(RegisterClassA(&window_class), 0);
            let handle = CreateWindowExA(
                Default::default(),
                PCSTR(WINDOW_CLASS_NAME.as_ptr()),
                PCSTR(window_name.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                Some(instance.into()),
                Some(window.as_mut() as *mut _ as _),
            )
            .expect("Failed to create window... PANIC!");
            window.dim = Window::get_client_dimensions(handle);
            window.buffer = Window::create_back_buffer(bitmap);
            window
        }
    }

    pub unsafe fn get_client_dimensions(window: HWND) -> WindowDim {
        let mut client_rect = RECT::default();
        let _ = GetClientRect(window, &mut client_rect);
        WindowDim {
            width: client_rect.right - client_rect.left,
            height: client_rect.bottom - client_rect.top,
        }
    }

    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTA;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).handle = window;
                (*this).dim = Window::get_client_dimensions(window);
                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
                DefWindowProcA(window, message, wparam, lparam)
            }
            WM_PAINT => {
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                let mut paint = PAINTSTRUCT::default();
                let device_ctx = BeginPaint(window, &mut paint);
                (*this).display_buffer_to_window(device_ctx);
                let _ = EndPaint(window, &paint);
                LRESULT(0)
            }
            WM_SIZE => {
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                (*this).dim = Window::get_client_dimensions(window);
                LRESULT(0)
            }
            WM_CLOSE => {
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                (*this).shutdown_requested = true;
                DefWindowProcA(window, message, wparam, lparam)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }

    pub fn display(&self) {
        unsafe {
            let device_ctx = GetDC(Some(self.handle));
            SetStretchBltMode(device_ctx, HALFTONE);
            self.display_buffer_to_window(device_ctx);
            ReleaseDC(Some(self.handle), device_ctx);
        }
    }

    pub unsafe fn display_buffer_to_window(&self, device_ctx: HDC) {
        let buffer_lock = self.buffer.bitmap.lock().unwrap();
        let data_ptr = buffer_lock.data.as_ref().unwrap().as_ptr() as *const std::ffi::c_void;
        let result = StretchDIBits(
            device_ctx,
            0,
            0,
            self.dim.width,
            self.dim.height,
            0,
            0,
            buffer_lock.width,
            buffer_lock.height,
            Some(data_ptr),
            &self.buffer.info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        if result == 0 {
            let error_code = GetLastError();
            if error_code.is_err() {
                eprintln!("StretchDIBits failed with error code: {}", error_code.0);
            }
        }
    }

    pub fn process_messages(&self) {
        let mut message = MSG::default();
        unsafe {
            while PeekMessageA(&mut message, Some(self.handle), 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
    }

    fn create_back_buffer(bitmap: Bitmap) -> Win32BackBuffer {
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.width,
                biHeight: -bitmap.height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            bmiColors: [RGBQUAD::default()],
        };
        Win32BackBuffer {
            bitmap: Arc::new(Mutex::new(bitmap)),
            info,
        }
    }
}
