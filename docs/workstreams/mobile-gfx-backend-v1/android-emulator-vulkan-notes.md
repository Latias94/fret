---
title: Mobile GFX Backend Selection v1 - Android Emulator Vulkan Notes
status: draft
date: 2026-02-13
platform: Android Emulator
---

# Android Emulator Vulkan notes

The Android Emulator *can* expose Vulkan, but the feature surface and stability vary by:

- Emulator version (gfxstream changes frequently),
- host OS and GPU backend (macOS uses Metal-based virtualization layers),
- AVD image / API level,
- driver translation (SwiftShader, MoltenVK, etc.).

For Fret’s purposes:

- Treat the emulator as **best-effort** for UI iteration and non-GPU smoke tests.
- Use **real Android devices** as the acceptance signal for Vulkan-first posture (M3).

## Quick checks

1) Confirm the device is reachable:

```bash
adb devices -l
```

2) Check guest Vulkan properties (best-effort; varies by system image):

```bash
adb -s <serial> shell getprop ro.hardware.vulkan
adb -s <serial> shell getprop ro.hardware.vulkan.level
adb -s <serial> shell getprop ro.hardware.vulkan.version
```

3) Capture the emulator-side warning signals (host log):

- If you see warnings like “missing required Vulkan features” (e.g. `VulkanVirtualQueue`), treat it as an
  emulator/virtualization limitation, not a framework contract violation.

## Launch knobs (host-side)

Typical graphics flags:

- `-gpu host`: use host GPU acceleration (preferred for Vulkan experiments; can still be unstable).
- `-gpu swiftshader_indirect`: software GPU path (often more stable, but may not represent real devices).

Example:

```bash
emulator @<avd> -gpu host
```

## Fret-specific guidance

Fret’s renderer currently requires:

- `wgpu::DownlevelFlags::VERTEX_STORAGE`

If the emulator reports a Vulkan adapter that does not satisfy required downlevel flags, Fret should fail fast by
default. For emulator-only iteration, you may opt into debug fallback attempts:

- `FRET_WGPU_ALLOW_FALLBACK=1` (debug builds only; should remain fail-fast in CI/release).

## Evidence

If emulator Vulkan is used for iteration, still archive the same fields as on real devices:

- selected backend,
- adapter name/vendor/device,
- driver/driver_info,
- downlevel flags and `is_webgpu_compliant`,
- init attempts (when fallback is enabled).

## References

- Android Emulator graphics acceleration docs (Vulkan notes are host-OS specific).
- Android Emulator release notes (gfxstream/Vulkan changes are often called out per version).

