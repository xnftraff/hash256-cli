# gpu-probe

Small Rust utility that detects GPU adapters on the current machine via
[`wgpu`](https://github.com/gfx-rs/wgpu). Cross-platform: on Linux/Windows it
sees NVIDIA/AMD/Intel through Vulkan and DX12, on macOS it sees Apple and eGPU
through Metal. Falls back to the GL backend on systems without Vulkan.

Used by the HASH256 miner to report which GPUs are available. Detection only —
it does not hash. A full GPU miner would need a compute shader (WGSL) or a
CUDA/OpenCL kernel on top of this.

## Build

```bash
cd gpu-probe
cargo build --release
```

The resulting binary is at `target/release/gpu-probe` (or `gpu-probe.exe` on
Windows). The Node miner auto-discovers it; you can also point at a custom
path with `GPU_PROBE=/path/to/gpu-probe npm start`.

## Usage

```bash
# Human-readable
./target/release/gpu-probe

# JSON (for piping into other tools)
./target/release/gpu-probe --format json

# Filter by backend: all (default) | primary | vulkan | dx12 | metal | gl
./target/release/gpu-probe --backends vulkan
```

## Exit codes

- `0` — at least one hardware GPU (discrete/integrated/virtual) detected
- `1` — only software/CPU fallback adapters present
- `2` — no adapters at all (no drivers installed, or headless server)

## JSON shape

```json
{
  "adapters": [
    {
      "name": "NVIDIA GeForce RTX 3070",
      "vendor": 4318,
      "vendor_name": "NVIDIA",
      "device": 9348,
      "device_type": "discrete",
      "backend": "vulkan",
      "driver": "NVIDIA",
      "driver_info": "550.120"
    }
  ],
  "has_hardware_gpu": true
}
```
