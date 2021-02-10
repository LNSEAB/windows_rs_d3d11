use bindings::{
    windows::win32::direct3d11::*, windows::win32::dxgi::*,
    windows::win32::windows_and_messaging::HWND,
};
use std::ptr::{null, null_mut};
use windows::Abi;
use windows::Interface;

struct Application {
    dc: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    rtv: ID3D11RenderTargetView,
}

impl Application {
    pub fn new() -> windows::Result<Self> {
        unsafe {
            let wnd = wita::WindowBuilder::new().title("windows-rs d3d11").build();
            let wnd_size = wnd.inner_size();
            let (device, dc) = {
                let mut device = None;
                let mut dc = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_HARDWARE,
                    0,
                    0,
                    [D3D_FEATURE_LEVEL::D3D_FEATURE_LEVEL_11_0].as_ptr(),
                    1,
                    D3D11_SDK_VERSION as _,
                    &mut device,
                    null_mut(),
                    &mut dc,
                )
                .ok()?;
                (device.unwrap(), dc.unwrap())
            };
            let dxgi_factory = {
                let mut dxgi_factory: Option<IDXGIFactory2> = None;
                CreateDXGIFactory1(&IDXGIFactory2::IID, dxgi_factory.set_abi())
                    .and_some(dxgi_factory)?
            };
            let swap_chain = {
                let mut swap_chain = None;
                dxgi_factory
                    .CreateSwapChainForHwnd(
                        &device,
                        HWND(wnd.raw_handle() as _),
                        &DXGI_SWAP_CHAIN_DESC1 {
                            width: wnd_size.width as _,
                            height: wnd_size.height as _,
                            format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
                            buffer_count: 2,
                            buffer_usage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                            sample_desc: DXGI_SAMPLE_DESC {
                                count: 1,
                                quality: 0,
                            },
                            swap_effect: DXGI_SWAP_EFFECT::DXGI_SWAP_EFFECT_FLIP_DISCARD,
                            ..Default::default()
                        },
                        null(),
                        None,
                        &mut swap_chain,
                    )
                    .and_some(swap_chain)?
            };
            let rtv = {
                let mut buffer: Option<ID3D11Texture2D> = None;
                let buffer = swap_chain
                    .GetBuffer(0, &ID3D11Texture2D::IID, buffer.set_abi())
                    .and_some(buffer)?;
                let mut rtv = None;
                device
                    .CreateRenderTargetView(buffer, null(), &mut rtv)
                    .and_some(rtv)?
            };
            Ok(Application {
                dc,
                swap_chain,
                rtv,
            })
        }
    }
}

impl wita::EventHandler for Application {
    fn idle(&mut self) {
        unsafe {
            self.dc
                .ClearRenderTargetView(&self.rtv, [0.0f32, 0.0, 0.3, 0.0].as_ptr());
            if let Err(e) = self.swap_chain.Present(0, 0).ok() {
                eprintln!("{:?}", e);
            }
        }
    }
}

fn main() {
    windows::initialize_sta().unwrap();
    wita::initialize::<Application>();
    wita::run(wita::RunType::Idle, Application::new().unwrap());
}
