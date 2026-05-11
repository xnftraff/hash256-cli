// gpu-probe: enumerate GPU adapters visible to wgpu and print them.
//
// Uses wgpu's cross-platform backend layer, so on a real machine this covers:
//   Vulkan   (Linux, Windows) -> NVIDIA, AMD, Intel
//   DX12     (Windows)        -> NVIDIA, AMD, Intel
//   Metal    (macOS, iOS)     -> Apple Silicon, AMD eGPU
//   GL       (fallback)       -> older drivers, software rasterizer
//
// Output formats:
//   - `text` (default): human-readable list
//   - `json`: machine-readable, suitable for consumption by Node/other tools
//
// Exit codes:
//   0 = at least one non-software GPU adapter found
//   1 = only software/CPU fallback adapters found
//   2 = no adapters at all

use clap::{Parser, ValueEnum};
use serde::Serialize;
use wgpu::{Backends, DeviceType, Instance, InstanceDescriptor};

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "gpu-probe",
    version,
    about = "Detect GPU adapters available on this system (via wgpu)."
)]
struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,

    /// Limit detection to a specific backend set.
    /// `all` (default), `vulkan`, `dx12`, `metal`, `gl`, `primary` (vk+dx12+metal).
    #[arg(long, default_value = "all")]
    backends: String,
}

#[derive(Serialize)]
struct AdapterInfo {
    name: String,
    vendor: u32,
    vendor_name: &'static str,
    device: u32,
    device_type: &'static str,
    backend: &'static str,
    driver: String,
    driver_info: String,
}

#[derive(Serialize)]
struct Report {
    adapters: Vec<AdapterInfo>,
    has_hardware_gpu: bool,
}

fn parse_backends(s: &str) -> Backends {
    match s.to_ascii_lowercase().as_str() {
        "all" => Backends::all(),
        "primary" => Backends::PRIMARY,
        "vulkan" | "vk" => Backends::VULKAN,
        "dx12" | "d3d12" => Backends::DX12,
        "metal" | "mtl" => Backends::METAL,
        "gl" | "opengl" | "gles" => Backends::GL,
        other => {
            eprintln!(
                "unknown backends filter `{}`, falling back to `all`",
                other
            );
            Backends::all()
        }
    }
}

fn device_type_str(t: DeviceType) -> &'static str {
    match t {
        DeviceType::Other => "other",
        DeviceType::IntegratedGpu => "integrated",
        DeviceType::DiscreteGpu => "discrete",
        DeviceType::VirtualGpu => "virtual",
        DeviceType::Cpu => "cpu",
    }
}

fn backend_str(b: wgpu::Backend) -> &'static str {
    match b {
        wgpu::Backend::Empty => "empty",
        wgpu::Backend::Vulkan => "vulkan",
        wgpu::Backend::Metal => "metal",
        wgpu::Backend::Dx12 => "dx12",
        wgpu::Backend::Gl => "gl",
        wgpu::Backend::BrowserWebGpu => "webgpu",
    }
}

// Best-effort PCI vendor decode for common GPU vendors.
fn vendor_name(id: u32) -> &'static str {
    match id {
        0x10DE => "NVIDIA",
        0x1002 | 0x1022 => "AMD",
        0x8086 => "Intel",
        0x106B => "Apple",
        0x5143 => "Qualcomm",
        0x13B5 => "ARM",
        0x1010 => "ImgTec",
        0 => "software",
        _ => "unknown",
    }
}

fn main() {
    let cli = Cli::parse();
    let backends = parse_backends(&cli.backends);

    let instance = Instance::new(InstanceDescriptor {
        backends,
        ..Default::default()
    });

    let adapters: Vec<_> = instance.enumerate_adapters(backends).into_iter().collect();

    let mut report = Report {
        adapters: Vec::with_capacity(adapters.len()),
        has_hardware_gpu: false,
    };

    for a in &adapters {
        let info = a.get_info();
        let is_hw = matches!(
            info.device_type,
            DeviceType::DiscreteGpu | DeviceType::IntegratedGpu | DeviceType::VirtualGpu
        );
        if is_hw {
            report.has_hardware_gpu = true;
        }
        report.adapters.push(AdapterInfo {
            name: info.name.clone(),
            vendor: info.vendor,
            vendor_name: vendor_name(info.vendor),
            device: info.device,
            device_type: device_type_str(info.device_type),
            backend: backend_str(info.backend),
            driver: info.driver.clone(),
            driver_info: info.driver_info.clone(),
        });
    }

    match cli.format {
        Format::Json => {
            let s = serde_json::to_string_pretty(&report).expect("serialize report");
            println!("{}", s);
        }
        Format::Text => {
            if report.adapters.is_empty() {
                println!("No GPU adapters detected.");
            } else {
                println!("Detected {} adapter(s):", report.adapters.len());
                for (i, a) in report.adapters.iter().enumerate() {
                    println!(
                        "  [{i}] {} ({}) via {}  type={}  vendor=0x{:04X}  device=0x{:04X}",
                        a.name, a.vendor_name, a.backend, a.device_type, a.vendor, a.device
                    );
                    if !a.driver.is_empty() || !a.driver_info.is_empty() {
                        println!("       driver: {} {}", a.driver, a.driver_info);
                    }
                }
            }
            println!();
            println!(
                "hardware GPU available: {}",
                if report.has_hardware_gpu { "yes" } else { "no" }
            );
        }
    }

    std::process::exit(if report.adapters.is_empty() {
        2
    } else if report.has_hardware_gpu {
        0
    } else {
        1
    });
}
