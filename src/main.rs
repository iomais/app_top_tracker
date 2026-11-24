#![windows_subsystem = "windows"]
use eframe::egui;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, CloseHandle, COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, SelectObject, SetDIBitsToDevice,
    SetPixel, SRCCOPY, StretchBlt, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HDC,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow, EnumWindows,
    GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    HICON, ICONINFO, IsWindowVisible, RegisterClassW, SetForegroundWindow, SetWindowPos, ShowWindow,
    WINDOW_EX_STYLE, GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, SW_HIDE, SW_RESTORE, SW_SHOW, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
    WM_USER, WNDCLASSW, WS_EX_TOOLWINDOW,
};

const AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const WM_TRAYICON: u32 = WM_USER + 1;
const TRAY_UID: u32 = 1001;
const ICON_SIZE: usize = 32;

static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static TRAY_ACTIVE: AtomicBool = AtomicBool::new(false);
static TRAY_HWND: OnceLock<isize> = OnceLock::new();
static TRAY_HICON: OnceLock<isize> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
struct WindowItem {
    hwnd: isize,
    title: String,
    is_topmost: bool,
}

#[derive(Clone, Debug)]
struct AppGroup {
    process_name: String,
    windows: Vec<WindowItem>,
    expanded: bool,
}

impl AppGroup {
    fn any_topmost(&self) -> bool {
        self.windows.iter().any(|w| w.is_topmost)
    }

    fn equivalent_content(&self, other: &Self) -> bool {
        self.process_name == other.process_name && self.windows == other.windows
    }
}

pub struct TopAppTracker {
    groups: Vec<AppGroup>,
    last_refresh: Instant,
}

impl Default for TopAppTracker {
    fn default() -> Self {
        let mut tracker = Self {
            groups: Vec::new(),
            last_refresh: Instant::now(),
        };
        tracker.refresh_apps();
        tracker
    }
}

impl Drop for TopAppTracker {
    fn drop(&mut self) {
        remove_tray_icon_now();
        if let Some(&hicon_raw) = TRAY_HICON.get() {
            if hicon_raw != 0 {
                unsafe {
                    let _ = DestroyIcon(HICON(hicon_raw));
                }
            }
        }
    }
}

fn build_pin_window_icon_rgba() -> Vec<u8> {
    let w = ICON_SIZE as i32;
    let h = ICON_SIZE as i32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];

    let set_px = |buf: &mut [u8], x: i32, y: i32, r: u8, g: u8, b: u8, a: u8| {
        if x >= 0 && x < w && y >= 0 && y < h {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx] = r;
            buf[idx + 1] = g;
            buf[idx + 2] = b;
            buf[idx + 3] = a;
        }
    };

    for y in 10..=27 {
        for x in 4..=24 {
            let is_border = x == 4 || x == 24 || y == 10 || y == 27;
            let is_titlebar = y > 10 && y <= 14 && x > 4 && x < 24;
            if is_border {
                set_px(&mut pixels, x, y, 30, 86, 160, 255);
            } else if is_titlebar {
                set_px(&mut pixels, x, y, 52, 125, 215, 255);
            } else {
                set_px(&mut pixels, x, y, 242, 246, 252, 255);
            }
        }
    }

    for x in 7..=16 {
        set_px(&mut pixels, x, 18, 160, 175, 195, 255);
        set_px(&mut pixels, x, 22, 160, 175, 195, 255);
    }

    for step in 0..=8 {
        let px = 19 - step;
        let py = 12 + step;
        set_px(&mut pixels, px, py, 95, 105, 120, 255);
        set_px(&mut pixels, px + 1, py, 130, 140, 155, 255);
    }

    let cx = 22.0f32;
    let cy = 8.0f32;
    for y in 3..=13 {
        for x in 17..=27 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= 4.6 {
                if dx < -0.5 && dy < -0.5 {
                    set_px(&mut pixels, x, y, 255, 140, 140, 255);
                } else {
                    set_px(&mut pixels, x, y, 215, 45, 55, 255);
                }
            } else if dist <= 5.4 {
                set_px(&mut pixels, x, y, 125, 20, 28, 255);
            }
        }
    }

    for step in -2..=2 {
        let px = 19 + step;
        let py = 11 - step;
        set_px(&mut pixels, px, py, 200, 40, 50, 255);
        set_px(&mut pixels, px + 1, py + 1, 140, 25, 35, 255);
    }

    pixels
}

fn create_win32_hicon_from_rgba(rgba: &[u8], width: i32, height: i32) -> isize {
    if let Some(&existing) = TRAY_HICON.get() {
        if existing != 0 {
            return existing;
        }
    }

    unsafe {
        let hdc_screen: HDC = GetDC(None);
        let hdc_color = CreateCompatibleDC(hdc_screen);
        let hdc_mask = CreateCompatibleDC(hdc_screen);

        let mut bgra = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let src_y = height - 1 - y;
                let src_idx = ((src_y * width + x) * 4) as usize;
                let dst_idx = ((y * width + x) * 4) as usize;
                bgra[dst_idx] = rgba[src_idx + 2];
                bgra[dst_idx + 1] = rgba[src_idx + 1];
                bgra[dst_idx + 2] = rgba[src_idx];
                bgra[dst_idx + 3] = rgba[src_idx + 3];
            }
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let hbm_color = CreateBitmap(width, height, 1, 32, None);
        let hbm_mask = CreateBitmap(width, height, 1, 1, None);

        let old_color = SelectObject(hdc_color, hbm_color);
        let old_mask = SelectObject(hdc_mask, hbm_mask);

        let _ = SetDIBitsToDevice(
            hdc_color,
            0,
            0,
            width as u32,
            height as u32,
            0,
            0,
            0,
            height as u32,
            bgra.as_ptr() as *const _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        for y in 0..height {
            for x in 0..width {
                let alpha = rgba[((y * width + x) * 4 + 3) as usize];
                let color_ref = if alpha == 0 {
                    0x00FF_FFFF
                } else {
                    0x0000_0000
                };
                SetPixel(hdc_mask, x, y, COLORREF(color_ref));
            }
        }

        SelectObject(hdc_color, old_color);
        SelectObject(hdc_mask, old_mask);

        let mut icon_info = ICONINFO {
            fIcon: BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let hicon = CreateIconIndirect(&mut icon_info).unwrap_or_default();

        let _ = DeleteObject(hbm_color);
        let _ = DeleteObject(hbm_mask);
        let _ = DeleteDC(hdc_color);
        let _ = DeleteDC(hdc_mask);
        let _ = StretchBlt(hdc_screen, 0, 0, 0, 0, hdc_screen, 0, 0, 0, 0, SRCCOPY);

        let raw = hicon.0;
        let _ = TRAY_HICON.set(raw);
        raw
    }
}

fn find_own_main_window() -> Option<HWND> {
    unsafe {
        let current_pid = GetCurrentProcessId();
        let mut handles: Vec<HWND> = Vec::new();
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut handles as *mut _ as isize),
        );

        for hwnd in handles {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == current_pid && GetWindowTextLengthW(hwnd) > 0 {
                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                if (ex_style & WS_EX_TOOLWINDOW.0) == 0 {
                    return Some(hwnd);
                }
            }
        }
        None
    }
}

fn remove_tray_icon_now() {
    if !TRAY_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    if let Some(&stored) = TRAY_HWND.get() {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = HWND(stored);
            nid.uID = TRAY_UID;
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }
}

fn restore_main_window_now() {
    let mut raw = MAIN_HWND.load(Ordering::SeqCst);
    if raw == 0 {
        if let Some(h) = find_own_main_window() {
            raw = h.0;
            MAIN_HWND.store(raw, Ordering::SeqCst);
        }
    }

    if raw != 0 {
        let hwnd = HWND(raw);
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetWindowPos(
                hwnd,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            let _ = SetForegroundWindow(hwnd);
        }
    }

    remove_tray_icon_now();
}

impl TopAppTracker {
    fn collect_groups_snapshot(prev_expanded: &HashMap<String, bool>) -> Vec<AppGroup> {
        let mut grouped: BTreeMap<String, Vec<WindowItem>> = BTreeMap::new();

        unsafe {
            let mut handles: Vec<HWND> = Vec::new();
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut handles as *mut _ as isize),
            );

            for hwnd in handles {
                if !is_user_app(hwnd) {
                    continue;
                }

                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                let proc_name = get_short_process_name(pid);

                let raw_title = get_window_title(hwnd);
                let title = fix_arabic_for_egui(&raw_title);

                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                let is_topmost = (ex_style & 0x00000008) != 0;

                grouped.entry(proc_name).or_default().push(WindowItem {
                    hwnd: hwnd.0,
                    title,
                    is_topmost,
                });
            }
        }

        grouped
            .into_iter()
            .map(|(process_name, windows)| {
                let expanded = *prev_expanded.get(&process_name).unwrap_or(&false);
                AppGroup {
                    process_name,
                    windows,
                    expanded,
                }
            })
            .collect()
    }

    fn groups_changed(old: &[AppGroup], new: &[AppGroup]) -> bool {
        if old.len() != new.len() {
            return true;
        }
        old.iter()
            .zip(new.iter())
            .any(|(a, b)| !a.equivalent_content(b))
    }

    fn refresh_apps(&mut self) {
        self.last_refresh = Instant::now();

        let mut prev_expanded = HashMap::new();
        for g in &self.groups {
            prev_expanded.insert(g.process_name.clone(), g.expanded);
        }

        let new_groups = Self::collect_groups_snapshot(&prev_expanded);
        if Self::groups_changed(&self.groups, &new_groups) {
            self.groups = new_groups;
        }
    }

    fn set_hwnd_topmost(hwnd_val: isize, enable: bool) {
        let hwnd = HWND(hwnd_val);
        let flag = if enable {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        unsafe {
            let _ = SetWindowPos(hwnd, flag, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }

    fn toggle_window(&mut self, group_idx: usize, win_idx: usize) {
        if let Some(group) = self.groups.get_mut(group_idx) {
            if let Some(win) = group.windows.get_mut(win_idx) {
                let next_state = !win.is_topmost;
                Self::set_hwnd_topmost(win.hwnd, next_state);
                win.is_topmost = next_state;
            }
        }
    }

    fn toggle_group(&mut self, group_idx: usize) {
        if let Some(group) = self.groups.get_mut(group_idx) {
            let target_state = !group.any_topmost();
            for win in &mut group.windows {
                Self::set_hwnd_topmost(win.hwnd, target_state);
                win.is_topmost = target_state;
            }
        }
    }

    fn ensure_tray_hwnd(&self) -> Option<HWND> {
        if let Some(&stored) = TRAY_HWND.get() {
            return Some(HWND(stored));
        }

        unsafe {
            let hinstance = GetModuleHandleW(None).ok()?;
            let class_name: Vec<u16> = "AppTopTrackerTrayClass\0".encode_utf16().collect();

            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(class_name.as_ptr()),
                Default::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                hinstance,
                None,
            );

            if hwnd.0 == 0 {
                return None;
            }

            let _ = TRAY_HWND.set(hwnd.0);
            Some(hwnd)
        }
    }

    fn hide_to_tray(&self) {
        if MAIN_HWND.load(Ordering::SeqCst) == 0 {
            if let Some(h) = find_own_main_window() {
                MAIN_HWND.store(h.0, Ordering::SeqCst);
            }
        }

        let Some(tray_hwnd) = self.ensure_tray_hwnd() else {
            return;
        };

        let rgba = build_pin_window_icon_rgba();
        let hicon_raw = create_win32_hicon_from_rgba(&rgba, ICON_SIZE as i32, ICON_SIZE as i32);

        unsafe {
            if !TRAY_ACTIVE.swap(true, Ordering::SeqCst) {
                let mut nid = NOTIFYICONDATAW::default();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = tray_hwnd;
                nid.uID = TRAY_UID;
                nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
                nid.uCallbackMessage = WM_TRAYICON;
                nid.hIcon = HICON(hicon_raw);

                let tip = "App Top Tracker";
                for (i, ch) in tip.encode_utf16().take(127).enumerate() {
                    nid.szTip[i] = ch;
                }

                if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
                    TRAY_ACTIVE.store(false, Ordering::SeqCst);
                }
            }

            let raw = MAIN_HWND.load(Ordering::SeqCst);
            if raw != 0 {
                let _ = ShowWindow(HWND(raw), SW_HIDE);
            }
        }
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = (lparam.0 & 0xFFFF) as u32;
            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                restore_main_window_now();
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, _wparam, lparam),
    }
}

fn get_short_process_name(pid: u32) -> String {
    if pid == 0 {
        return "Unknown".to_string();
    }

    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
            Ok(h) => h,
            Err(_) => return format!("PID {}", pid),
        };

        let mut buf = [0u16; 260];
        let len = GetModuleBaseNameW(handle, None, &mut buf);
        let _ = CloseHandle(handle);

        if len == 0 {
            return format!("PID {}", pid);
        }

        let full_name = OsString::from_wide(&buf[..len as usize])
            .to_string_lossy()
            .into_owned();

        let short = full_name
            .strip_suffix(".exe")
            .or_else(|| full_name.strip_suffix(".EXE"))
            .unwrap_or(&full_name);

        short.to_string()
    }
}

fn arabic_glyph(ch: char, prev_connects_out: bool, next_connects_in: bool) -> (char, bool) {
    let table: &[(char, char, char, char, char, bool)] = &[
        ('ء', '\u{FE80}', '\u{FE80}', '\u{FE80}', '\u{FE80}', false),
        ('آ', '\u{FE81}', '\u{FE82}', '\u{FE81}', '\u{FE82}', false),
        ('أ', '\u{FE83}', '\u{FE84}', '\u{FE83}', '\u{FE84}', false),
        ('ؤ', '\u{FE85}', '\u{FE86}', '\u{FE85}', '\u{FE86}', false),
        ('إ', '\u{FE87}', '\u{FE88}', '\u{FE87}', '\u{FE88}', false),
        ('ئ', '\u{FE89}', '\u{FE8A}', '\u{FE8B}', '\u{FE8C}', true),
        ('ا', '\u{FE8D}', '\u{FE8E}', '\u{FE8D}', '\u{FE8E}', false),
        ('ب', '\u{FE8F}', '\u{FE90}', '\u{FE91}', '\u{FE92}', true),
        ('ة', '\u{FE93}', '\u{FE94}', '\u{FE93}', '\u{FE94}', false),
        ('ت', '\u{FE95}', '\u{FE96}', '\u{FE97}', '\u{FE98}', true),
        ('ث', '\u{FE99}', '\u{FE9A}', '\u{FE9B}', '\u{FE9C}', true),
        ('ج', '\u{FE9D}', '\u{FE9E}', '\u{FE9F}', '\u{FEA0}', true),
        ('ح', '\u{FEA1}', '\u{FEA2}', '\u{FEA3}', '\u{FEA4}', true),
        ('خ', '\u{FEA5}', '\u{FEA6}', '\u{FEA7}', '\u{FEA8}', true),
        ('د', '\u{FEA9}', '\u{FEAA}', '\u{FEA9}', '\u{FEAA}', false),
        ('ذ', '\u{FEAB}', '\u{FEAC}', '\u{FEAB}', '\u{FEAC}', false),
        ('ر', '\u{FEAD}', '\u{FEAE}', '\u{FEAD}', '\u{FEAE}', false),
        ('ز', '\u{FEAF}', '\u{FEB0}', '\u{FEAF}', '\u{FEB0}', false),
        ('س', '\u{FEB1}', '\u{FEB2}', '\u{FEB3}', '\u{FEB4}', true),
        ('ش', '\u{FEB5}', '\u{FEB6}', '\u{FEB7}', '\u{FEB8}', true),
        ('ص', '\u{FEB9}', '\u{FEBA}', '\u{FEBB}', '\u{FEBC}', true),
        ('ض', '\u{FEBD}', '\u{FEBE}', '\u{FEBF}', '\u{FEC0}', true),
        ('ط', '\u{FEC1}', '\u{FEC2}', '\u{FEC3}', '\u{FEC4}', true),
        ('ظ', '\u{FEC5}', '\u{FEC6}', '\u{FEC7}', '\u{FEC8}', true),
        ('ع', '\u{FEC9}', '\u{FECA}', '\u{FECB}', '\u{FECC}', true),
        ('غ', '\u{FECD}', '\u{FECE}', '\u{FECF}', '\u{FED0}', true),
        ('ف', '\u{FED1}', '\u{FED2}', '\u{FED3}', '\u{FED4}', true),
        ('ق', '\u{FED5}', '\u{FED6}', '\u{FED7}', '\u{FED8}', true),
        ('ك', '\u{FED9}', '\u{FEDA}', '\u{FEDB}', '\u{FEDC}', true),
        ('ل', '\u{FEDD}', '\u{FEDE}', '\u{FEDF}', '\u{FEE0}', true),
        ('م', '\u{FEE1}', '\u{FEE2}', '\u{FEE3}', '\u{FEE4}', true),
        ('ن', '\u{FEE5}', '\u{FEE6}', '\u{FEE7}', '\u{FEE8}', true),
        ('ه', '\u{FEE9}', '\u{FEEA}', '\u{FEEB}', '\u{FEEC}', true),
        ('و', '\u{FEED}', '\u{FEEE}', '\u{FEED}', '\u{FEEE}', false),
        ('ى', '\u{FEEF}', '\u{FEF0}', '\u{FEEF}', '\u{FEF0}', false),
        ('ي', '\u{FEF1}', '\u{FEF2}', '\u{FEF3}', '\u{FEF4}', true),
    ];

    if let Some((_, iso, end, init, med, can_connect_after)) =
        table.iter().find(|(base, _, _, _, _, _)| *base == ch)
    {
        let glyph = match (prev_connects_out, next_connects_in) {
            (false, false) => *iso,
            (true, false) => *end,
            (false, true) => *init,
            (true, true) => *med,
        };
        (glyph, *can_connect_after)
    } else {
        (ch, false)
    }
}

fn is_arabic_base(ch: char) -> bool {
    matches!(
        ch,
        'ء' | 'آ'
            | 'أ'
            | 'ؤ'
            | 'إ'
            | 'ئ'
            | 'ا'
            | 'ب'
            | 'ة'
            | 'ت'
            | 'ث'
            | 'ج'
            | 'ح'
            | 'خ'
            | 'د'
            | 'ذ'
            | 'ر'
            | 'ز'
            | 'س'
            | 'ش'
            | 'ص'
            | 'ض'
            | 'ط'
            | 'ظ'
            | 'ع'
            | 'غ'
            | 'ف'
            | 'ق'
            | 'ك'
            | 'ل'
            | 'م'
            | 'ن'
            | 'ه'
            | 'و'
            | 'ى'
            | 'ي'
    )
}

fn is_arabic_diacritic(ch: char) -> bool {
    ('\u{064B}'..='\u{065F}').contains(&ch) || ch == '\u{0670}'
}

fn fix_arabic_for_egui(input: &str) -> String {
    if !input.chars().any(is_arabic_base) {
        return input.to_string();
    }

    let chars: Vec<char> = input.chars().collect();
    let mut shaped: Vec<char> = Vec::with_capacity(chars.len());

    for i in 0..chars.len() {
        let c = chars[i];
        if is_arabic_base(c) {
            let prev_can_out = if i > 0 && is_arabic_base(chars[i - 1]) {
                arabic_glyph(chars[i - 1], false, false).1
            } else {
                false
            };

            let next_is_arabic = i + 1 < chars.len() && is_arabic_base(chars[i + 1]);
            let (g, _) = arabic_glyph(c, prev_can_out, next_is_arabic);
            shaped.push(g);
        } else {
            shaped.push(c);
        }
    }

    let is_ar_token_char = |ch: char| {
        is_arabic_base(ch) || is_arabic_diacritic(ch) || ('\u{FE70}'..='\u{FEFF}').contains(&ch)
    };

    let mut out = String::with_capacity(shaped.len());
    let mut i = 0;

    while i < shaped.len() {
        if is_ar_token_char(shaped[i]) {
            let start = i;
            while i < shaped.len()
                && (is_ar_token_char(shaped[i])
                    || (shaped[i] == ' '
                        && i + 1 < shaped.len()
                        && is_ar_token_char(shaped[i + 1])))
            {
                i += 1;
            }
            let reversed_seg: String = shaped[start..i].iter().rev().collect();
            out.push_str(&reversed_seg);
        } else {
            out.push(shaped[i]);
            i += 1;
        }
    }

    out
}

fn is_user_app(hwnd: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        if rect.right - rect.left == 0 || rect.bottom - rect.top == 0 {
            return false;
        }

        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
            return false;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return false;
        }

        true
    }
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; (length + 1) as usize];
        GetWindowTextW(hwnd, &mut buffer);
        OsString::from_wide(&buffer[..length as usize])
            .to_string_lossy()
            .into_owned()
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let handles = &mut *(lparam.0 as *mut Vec<HWND>);
    handles.push(hwnd);
    BOOL(1)
}

fn setup_arabic_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
        r"C:\Windows\Fonts\arial.ttf",
    ];

    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "arabic_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "arabic_font".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

impl eframe::App for TopAppTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if MAIN_HWND.load(Ordering::SeqCst) == 0 {
            if let Some(h) = find_own_main_window() {
                MAIN_HWND.store(h.0, Ordering::SeqCst);
            }
        }

        if self.last_refresh.elapsed() >= AUTO_REFRESH_INTERVAL {
            self.refresh_apps();
        }
        ctx.request_repaint_after(Duration::from_millis(400));

        let mut hide_clicked = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Apps");
                if ui.button("Refresh").clicked() {
                    self.refresh_apps();
                }
                if ui.button("Tray").clicked() {
                    hide_clicked = true;
                }
            });

            ui.separator();
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut group_toggle: Option<usize> = None;
                let mut expand_toggle: Option<usize> = None;
                let mut win_toggle: Option<(usize, usize)> = None;

                for (g_idx, group) in self.groups.iter().enumerate() {
                    let has_multiple = group.windows.len() > 1;
                    let is_top = group.any_topmost();

                    ui.horizontal(|ui| {
                        if has_multiple {
                            let arrow = if group.expanded { "▼" } else { "▶" };
                            if ui.small_button(arrow).clicked() {
                                expand_toggle = Some(g_idx);
                            }
                        } else {
                            ui.add_space(22.0);
                        }

                        let pin_btn = if is_top {
                            egui::Button::new("📌").fill(egui::Color32::from_rgb(50, 120, 200))
                        } else {
                            egui::Button::new("📍")
                        };

                        if ui.add(pin_btn).clicked() {
                            group_toggle = Some(g_idx);
                        }

                        if has_multiple {
                            ui.label(format!("{} ({})", group.process_name, group.windows.len()));
                        } else {
                            ui.label(&group.process_name);
                        }
                    });

                    if has_multiple && group.expanded {
                        ui.indent(format!("group_indent_{}", g_idx), |ui| {
                            for (w_idx, win) in group.windows.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let w_btn = if win.is_topmost {
                                        egui::Button::new("📌")
                                            .fill(egui::Color32::from_rgb(50, 120, 200))
                                    } else {
                                        egui::Button::new("📍")
                                    };

                                    if ui.add(w_btn).clicked() {
                                        win_toggle = Some((g_idx, w_idx));
                                    }

                                    let short_title: String = win.title.chars().take(36).collect();
                                    let display_title = if win.title.chars().count() > 36 {
                                        format!("{}…", short_title)
                                    } else {
                                        short_title
                                    };
                                    ui.label(display_title);
                                });
                            }
                        });
                    }

                    ui.separator();
                }

                if let Some(g_idx) = expand_toggle {
                    if let Some(group) = self.groups.get_mut(g_idx) {
                        group.expanded = !group.expanded;
                    }
                }
                if let Some(g_idx) = group_toggle {
                    self.toggle_group(g_idx);
                }
                if let Some((g_idx, w_idx)) = win_toggle {
                    self.toggle_window(g_idx, w_idx);
                }
            });
        });

        if hide_clicked {
            self.hide_to_tray();
        }
    }
}

fn main() -> eframe::Result<()> {
    let icon_rgba = build_pin_window_icon_rgba();
    let icon_data = egui::IconData {
        rgba: icon_rgba,
        width: ICON_SIZE as u32,
        height: ICON_SIZE as u32,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 420.0])
            .with_icon(icon_data),
        ..Default::default()
    };
    eframe::run_native(
        "App Top Tracker",
        options,
        Box::new(|cc| {
            setup_arabic_font(&cc.egui_ctx);
            Box::new(TopAppTracker::default())
        }),
    )
}