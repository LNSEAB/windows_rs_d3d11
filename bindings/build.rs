fn main() {
    windows::build!(
        windows::win32::direct3d11::*,
        windows::win32::dxgi::*,
        windows::win32::windows_and_messaging::*
    );
}