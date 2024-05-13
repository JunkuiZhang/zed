use windows::{
    core::*,
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
                D3D11_CREATE_DEVICE_DEBUG, D3D11_SDK_VERSION, D3D11_VIEWPORT,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGIDevice, IDXGIFactory1, IDXGIFactory2, IDXGISwapChain1, DXGI_MWA_NO_ALT_ENTER,
                DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    },
};

pub(crate) struct D3D11Renderer {
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    rtv: ID3D11RenderTargetView,
}

impl D3D11Renderer {
    pub(crate) fn new(hwnd: HWND) -> Self {
        unsafe {
            let mut device = None;
            let mut immediate_context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_DEBUG,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut immediate_context),
            )
            .unwrap();
            let device = device.unwrap();
            let immediate_context = immediate_context.unwrap();
            let dxgi_factory: IDXGIFactory2 = {
                let dxgi_device = device.cast::<IDXGIDevice>().unwrap();
                let adapter = dxgi_device.GetAdapter().unwrap();
                adapter.GetParent().unwrap()
            };
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: 1,
                Height: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 1,
                ..Default::default()
            };
            let swap_chain = dxgi_factory
                .CreateSwapChainForHwnd(&device, hwnd, &desc, None, None)
                .unwrap();
            dxgi_factory
                .MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER)
                .unwrap();
            let back_buffer: ID3D11Texture2D = swap_chain.GetBuffer(0).unwrap();
            let mut rtv = None;
            device
                .CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))
                .unwrap();
            immediate_context.OMSetRenderTargets(Some(&[rtv.clone()]), None);
            let vp = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: 1.0,
                Height: 1.0,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };
            immediate_context.RSSetViewports(Some(&[vp]));

            Self {
                context: immediate_context,
                swap_chain,
                rtv: rtv.unwrap(),
            }
        }
    }

    pub(crate) fn draw(&mut self) {
        unsafe {
            self.context
                .ClearRenderTargetView(&self.rtv, &[0.7, 0.5, 0.2, 1.0]);
            self.swap_chain.Present(1, 0).ok().unwrap();
        }
    }
}
