use bindings::{
    windows::win32::direct3d11::*, windows::win32::dxgi::*,
    windows::win32::windows_and_messaging::HWND,
    windows::win32::system_services::*,
};
use std::ptr::{null, null_mut};
use windows::Abi;
use windows::Interface;

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

struct Application {
    wnd: wita::Window,
    dc: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    rtv: ID3D11RenderTargetView,
    vertex_buffer: ID3D11Buffer,
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
}

impl Application {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let wnd = wita::WindowBuilder::new()
                .title("windows-rs d3d11")
                .build()?;
            let wnd_size = wnd.inner_size();
            let (device, dc) = {
                let mut device = None;
                let mut dc = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_HARDWARE,
                    0,
                    D3D11_CREATE_DEVICE_FLAG(0),
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
            let vertex_buffer = {
                let vertices = [
                    Vertex {
                        position: [-0.8, -0.8, 0.0],
                        color: [1.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.0, 0.8, 0.0],
                        color: [0.0, 1.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.8, -0.8, 0.0],
                        color: [0.0, 0.0, 1.0, 1.0],
                    },
                ];
                let mut vertex_buffer = None;
                device.CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        byte_width: (std::mem::size_of::<Vertex>() * 3) as _,
                        usage: D3D11_USAGE::D3D11_USAGE_DEFAULT,
                        bind_flags: D3D11_BIND_FLAG::D3D11_BIND_VERTEX_BUFFER.0 as _,
                        ..Default::default()
                    },
                    &D3D11_SUBRESOURCE_DATA {
                        p_sys_mem: vertices.as_ptr() as _,
                        ..Default::default()
                    },
                    &mut vertex_buffer
                ).and_some(vertex_buffer)?
            };
            let (vs, ps, input_layout) = {
                let vs_blob = include_bytes!("triangle.vs");
                let ps_blob = include_bytes!("triangle.ps");
                let vs = {
                    let mut vs = None;
                    device.CreateVertexShader(vs_blob.as_ptr() as _, vs_blob.len() as _, None, &mut vs).and_some(vs)?
                };
                let ps = {
                    let mut ps = None;
                    device.CreatePixelShader(ps_blob.as_ptr() as _, ps_blob.len() as _, None, &mut ps).and_some(ps)?
                };
                let position_name = std::ffi::CString::new("POSITION").unwrap();
                let color_name = std::ffi::CString::new("COLOR").unwrap();
                let descs = [
                    D3D11_INPUT_ELEMENT_DESC {
                        semantic_name: PSTR(position_name.as_ptr() as _),
                        semantic_index: 0,
                        format: DXGI_FORMAT::DXGI_FORMAT_R32G32B32_FLOAT,
                        input_slot: 0,
                        aligned_byte_offset: 0,
                        input_slot_class: D3D11_INPUT_CLASSIFICATION::D3D11_INPUT_PER_VERTEX_DATA,
                        instance_data_step_rate: 0,
                    },
                    D3D11_INPUT_ELEMENT_DESC {
                        semantic_name: PSTR(color_name.as_ptr() as _),
                        semantic_index: 0,
                        format: DXGI_FORMAT::DXGI_FORMAT_R32G32B32A32_FLOAT,
                        input_slot: 0,
                        aligned_byte_offset: D3D11_APPEND_ALIGNED_ELEMENT,
                        input_slot_class: D3D11_INPUT_CLASSIFICATION::D3D11_INPUT_PER_VERTEX_DATA,
                        instance_data_step_rate: 0,
                    },
                ];
                let input_layout = {
                    let mut input_layout = None;
                    device.CreateInputLayout(
                        descs.as_ptr(),
                        descs.len() as _,
                        vs_blob.as_ptr() as _,
                        vs_blob.len() as _,
                        &mut input_layout
                    ).and_some(input_layout)?
                };
                (vs, ps, input_layout)
            };
            Ok(Application {
                wnd,
                dc,
                swap_chain,
                rtv,
                vertex_buffer,
                vs,
                ps,
                input_layout,
            })
        }
    }
}

impl wita::EventHandler for Application {
    fn idle(&mut self) {
        let wnd_size = self.wnd.inner_size();
        unsafe {
            self.dc
                .ClearRenderTargetView(&self.rtv, [0.0f32, 0.0, 0.3, 0.0].as_ptr());
            self.dc.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.dc.IASetInputLayout(&self.input_layout);
            self.dc.IASetVertexBuffers(0, 1, [Some(self.vertex_buffer.clone())].as_mut_ptr(), [std::mem::size_of::<Vertex>() as u32].as_ptr(), [0].as_ptr());
            self.dc.OMSetRenderTargets(1, [Some(self.rtv.clone())].as_mut_ptr(), None);
            self.dc.VSSetShader(&self.vs, null_mut(), 0);
            self.dc.PSSetShader(&self.ps, null_mut(), 0);
            self.dc.RSSetViewports(1, [
                D3D11_VIEWPORT {
                    width: wnd_size.width as _,
                    height: wnd_size.height as _,
                    max_depth: 1.0,
                    ..Default::default()
                }
            ].as_ptr());
            self.dc.Draw(3, 0);
            if let Err(e) = self.swap_chain.Present(0, 0).ok() {
                eprintln!("{:?}", e);
            }
        }
    }
}

fn main() {
    wita::run(wita::RunType::Idle, Application::new).unwrap();
}
