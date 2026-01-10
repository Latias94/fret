import base64
import json
import os
import struct
import traceback

import renderdoc as rd


REQ_PATH = "fret_dump_pass_state_json.request.json"
RESP_PATH = "fret_dump_pass_state_json.response.json"


def write_response(path: str, obj) -> None:
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False)


def is_null_resource_id(rid) -> bool:
    try:
        if rid == rd.ResourceId():
            return True
    except Exception:
        pass

    try:
        return int(rid) == 0
    except Exception:
        try:
            return int(rid.value) == 0
        except Exception:
            return False


def extract_resource_id(obj):
    if obj is None:
        return None
    if hasattr(obj, "resourceId"):
        return obj.resourceId
    if hasattr(obj, "resource"):
        return obj.resource
    return None


def try_resource_name(controller, rid) -> str:
    try:
        if hasattr(controller, "GetResourceDescription"):
            desc = controller.GetResourceDescription(rid)
            if desc is None:
                return ""
            return str(getattr(desc, "name", "") or "")
    except Exception:
        pass
    try:
        if hasattr(controller, "GetResources"):
            for desc in controller.GetResources():
                try:
                    if int(getattr(desc, "resourceId", 0)) == int(rid):
                        return str(getattr(desc, "name", "") or "")
                except Exception:
                    pass
    except Exception:
        pass
    return ""


def try_get_buffer_data(controller, rid, offset: int, size: int):
    try:
        if hasattr(controller, "GetBufferData"):
            return controller.GetBufferData(rid, int(offset), int(size))
    except Exception:
        pass
    return None


def flatten_actions(actions):
    out = []
    for a in actions:
        out.append(a)
        out.extend(flatten_actions(a.children))
    return out


def marker_path_join(marker_path) -> str:
    if not marker_path:
        return ""
    return "/".join([str(x) for x in marker_path])


def normalize(s: str, case_sensitive: bool) -> str:
    if s is None:
        return ""
    if case_sensitive:
        return str(s)
    return str(s).lower()


def iter_actions(structured_file, actions, marker_stack, out, req):
    marker_contains = normalize(req["marker_contains"], bool(req.get("case_sensitive", False)))
    name_contains = normalize(req.get("name_contains", ""), bool(req.get("case_sensitive", False)))
    only_drawcalls = bool(req.get("only_drawcalls", True))
    max_results = int(req.get("max_results", 200))

    for a in actions:
        name = ""
        try:
            name = a.GetName(structured_file)
        except Exception:
            try:
                name = a.GetName(None)
            except Exception:
                try:
                    name = a.customName
                except Exception:
                    name = ""
        flags = a.flags

        effective_marker_path = list(marker_stack)
        if flags & rd.ActionFlags.PushMarker:
            effective_marker_path.append(str(name))
        joined_marker_path = marker_path_join(effective_marker_path)

        matched = True
        if marker_contains and marker_contains not in normalize(joined_marker_path, req.get("case_sensitive", False)):
            matched = False
        if name_contains and name_contains not in normalize(str(name), req.get("case_sensitive", False)):
            matched = False
        if only_drawcalls:
            matched = matched and bool(
                (flags & rd.ActionFlags.Drawcall)
                or (flags & rd.ActionFlags.Dispatch)
                or (flags & rd.ActionFlags.MeshDispatch)
                or (flags & rd.ActionFlags.DispatchRay)
            )

        if matched:
            out.append(
                {
                    "event_id": int(a.eventId),
                    "name": str(name),
                    "marker_path": joined_marker_path,
                    "flags": int(flags),
                }
            )
            if max_results is not None and len(out) >= max_results:
                return

        if flags & rd.ActionFlags.PushMarker:
            marker_stack.append(str(name))
            iter_actions(structured_file, a.children, marker_stack, out, req)
            marker_stack.pop()
        else:
            iter_actions(structured_file, a.children, marker_stack, out, req)

        if max_results is not None and len(out) >= max_results:
            return


def decode_viewport_uniform(data: bytes):
    # Rust: ViewportUniform (repr(C))
    #   viewport_size: [f32; 2]
    #   clip_head: u32
    #   clip_count: u32
    #   output_is_srgb: u32
    #   _pad: [u32; 3]
    #   mask_viewport_origin: [f32; 2]
    #   mask_viewport_size: [f32; 2]
    if data is None or len(data) < 48:
        return None

    f32 = lambda off: struct.unpack_from("<f", data, off)[0]
    u32 = lambda off: struct.unpack_from("<I", data, off)[0]

    return {
        "viewport_size": [f32(0), f32(4)],
        "clip_head": int(u32(8)),
        "clip_count": int(u32(12)),
        "output_is_srgb": int(u32(16)),
        "mask_viewport_origin": [f32(32), f32(36)],
        "mask_viewport_size": [f32(40), f32(44)],
    }


def decode_viewport_uniform_table(data: bytes, stride: int, count: int):
    out = []
    if data is None or stride <= 0:
        return out
    for i in range(0, count):
        off = i * stride
        if off + 48 > len(data):
            break
        decoded = decode_viewport_uniform(data[off : off + 48])
        if decoded is None:
            continue
        decoded["index"] = int(i)
        decoded["byte_offset"] = int(off)
        out.append(decoded)
    return out


def decode_clip_mask_params(data: bytes):
    # WGSL: struct Params { dst_size: vec2<f32>, _pad0: vec2<f32> };
    if data is None or len(data) < 16:
        return None
    f32 = lambda off: struct.unpack_from("<f", data, off)[0]
    return {"dst_size": [f32(0), f32(4)]}


def decode_scale_params(data: bytes):
    # WGSL:
    # struct ScaleParams {
    #   scale: u32,
    #   _pad0: u32,
    #   src_origin: vec2<u32>,
    #   dst_origin: vec2<u32>,
    #   _pad1: u32,
    #   _pad2: u32,
    # };
    if data is None or len(data) < 16:
        return None
    u32 = lambda off: struct.unpack_from("<I", data, off)[0]
    if len(data) < 32:
        return {"scale": int(u32(0))}
    return {
        "scale": int(u32(0)),
        "src_origin": [int(u32(8)), int(u32(12))],
        "dst_origin": [int(u32(16)), int(u32(20))],
    }


def decode_color_adjust_params(data: bytes):
    # WGSL: struct Params { saturation: f32, brightness: f32, contrast: f32, _pad: f32 };
    if data is None or len(data) < 16:
        return None
    f32 = lambda off: struct.unpack_from("<f", data, off)[0]
    return {
        "saturation": f32(0),
        "brightness": f32(4),
        "contrast": f32(8),
    }


def decode_clip_rrect_entries(data: bytes, count: int):
    # ClipRRectUniform is 16 f32 = 64 bytes.
    if data is None:
        return []
    entry_size = 64
    out = []
    for i in range(0, min(count, len(data) // entry_size)):
        off = i * entry_size
        floats = struct.unpack_from("<" + "f" * 16, data, off)
        rect = floats[0:4]
        radii = floats[4:8]
        inv0 = floats[8:12]
        inv1 = floats[12:16]
        inv0_w_bits = struct.unpack("<I", struct.pack("<f", inv0[3]))[0]
        out.append(
            {
                "index": int(i),
                "rect": list(rect),
                "corner_radii": list(radii),
                "inv0": list(inv0),
                "inv1": list(inv1),
                "next_index_bits": int(inv0_w_bits),
            }
        )
    return out


def try_texture_desc(controller, rid):
    try:
        if hasattr(controller, "GetTextureDescription"):
            desc = controller.GetTextureDescription(rid)
            if desc is None:
                return None
            fmt = getattr(desc, "format", None)
            fmt_str = str(fmt) if fmt is not None else ""
            return {
                "width": int(getattr(desc, "width", 0)),
                "height": int(getattr(desc, "height", 0)),
                "depth": int(getattr(desc, "depth", 0)),
                "mips": int(getattr(desc, "mips", 0)),
                "arrays": int(getattr(desc, "arraysize", 0)),
                "samples": int(getattr(desc, "msSamp", 0)),
                "format": fmt_str,
            }
    except Exception:
        pass
    return None


def try_viewport_scissor(pipe):
    out = {"viewports": [], "scissors": []}
    count = 1
    try:
        if hasattr(pipe, "GetNumViewports"):
            count = max(1, int(pipe.GetNumViewports()))
    except Exception:
        count = 1

    for i in range(0, count):
        try:
            if hasattr(pipe, "GetViewport"):
                vp = pipe.GetViewport(int(i))
                if vp is not None:
                    out["viewports"].append(
                        {
                            "index": int(i),
                            "x": float(getattr(vp, "x", 0.0)),
                            "y": float(getattr(vp, "y", 0.0)),
                            "width": float(getattr(vp, "width", 0.0)),
                            "height": float(getattr(vp, "height", 0.0)),
                            "min_depth": float(getattr(vp, "minDepth", 0.0)),
                            "max_depth": float(getattr(vp, "maxDepth", 1.0)),
                        }
                    )
        except Exception:
            pass
        try:
            if hasattr(pipe, "GetScissor"):
                sc = pipe.GetScissor(int(i))
                if sc is not None:
                    out["scissors"].append(
                        {
                            "index": int(i),
                            "x": int(getattr(sc, "x", 0)),
                            "y": int(getattr(sc, "y", 0)),
                            "width": int(getattr(sc, "width", 0)),
                            "height": int(getattr(sc, "height", 0)),
                        }
                    )
        except Exception:
            pass
    return out


def choose_uniform_entry_for_clip_mask(entries, clip_mask_dst_size):
    if not entries or clip_mask_dst_size is None:
        return None
    dst_w = float(clip_mask_dst_size[0] or 0.0)
    dst_h = float(clip_mask_dst_size[1] or 0.0)
    if dst_w <= 0.0 or dst_h <= 0.0:
        return None

    # For clip-mask generation, the output target may be full/half/quarter resolution of the
    # mask viewport rect. We try to find the viewport uniform entry whose
    # `mask_viewport_size / dst_size` is closest to a power-of-two scale (1/2/4).
    candidates = []
    for e in entries:
        try:
            if int(e.get("clip_count", 0)) <= 0:
                continue
            msz = e.get("mask_viewport_size", None)
            if not msz:
                continue
            mw = float(msz[0] or 0.0)
            mh = float(msz[1] or 0.0)
            if mw <= 0.0 or mh <= 0.0:
                continue
            sx = mw / dst_w
            sy = mh / dst_h
            expected = [1.0, 2.0, 4.0]
            sx_err = min([abs(sx - v) for v in expected])
            sy_err = min([abs(sy - v) for v in expected])
            err = sx_err + sy_err
            candidates.append((err, e))
        except Exception:
            pass

    if not candidates:
        return None
    candidates.sort(key=lambda t: t[0])
    best_err, best = candidates[0]
    return {"entry": best, "score": float(best_err)}


def dump_event(controller, event_id: int, req):
    controller.SetFrameEvent(int(event_id), True)
    pipe = controller.GetPipelineState()
    marker_contains = str(req.get("marker_contains", "") or "").lower()

    # Resource list (best-effort): used to locate the uniform/clip buffers by name.
    resources = []
    resource_descs = None
    if hasattr(controller, "GetResources"):
        try:
            resource_descs = controller.GetResources()
            for desc in resource_descs:
                try:
                    resources.append(
                        {
                            "resource_id": int(getattr(desc, "resourceId", 0)),
                            "name": str(getattr(desc, "name", "") or ""),
                            "type": str(getattr(desc, "type", "") or ""),
                        }
                    )
                except Exception:
                    pass
        except Exception:
            resources = []
            resource_descs = None

    outputs = []
    for i, br in enumerate(pipe.GetOutputTargets()):
        rid = extract_resource_id(br)
        if rid is None or is_null_resource_id(rid):
            continue
        outputs.append(
            {
                "index": int(i),
                "resource_id": int(rid),
                "resource_name": try_resource_name(controller, rid),
                "texture": try_texture_desc(controller, rid),
            }
        )

    depth = None
    br = pipe.GetDepthTarget()
    rid = extract_resource_id(br)
    if rid is not None and not is_null_resource_id(rid):
        depth = {
            "resource_id": int(rid),
            "resource_name": try_resource_name(controller, rid),
        }

    # Try to locate the uniform buffer and clip stack via reflection and bound resources.
    stage_dumps = []
    uniform_decoded = []
    uniform_raw = []

    dump_uniform_bytes = bool(req.get("dump_uniform_bytes", True))
    dump_clip_count = int(req.get("dump_clip_stack_entries", 64))

    # ShaderStage.Fragment is not consistently present; Pixel is the common alias.
    stage_pixel = getattr(rd.ShaderStage, "Pixel", getattr(rd.ShaderStage, "Fragment", None))
    stages_to_check = [rd.ShaderStage.Vertex]
    if stage_pixel is not None:
        stages_to_check.append(stage_pixel)
    stages_to_check.append(rd.ShaderStage.Compute)

    for stage in stages_to_check:
        try:
            shader = pipe.GetShader(stage)
        except Exception:
            shader = rd.ResourceId.Null()

        if shader == rd.ResourceId.Null():
            continue

        stage_name = str(stage)
        stage_info = {
            "stage": stage_name,
            "shader_resource_id": str(shader),
            "shader_name": try_resource_name(controller, shader),
            "entry_point": str(pipe.GetShaderEntryPoint(stage) or ""),
            "constant_buffers": [],
            "readonly_resources": [],
            "readwrite_resources": [],
        }

        try:
            refl = pipe.GetShaderReflection(stage)
        except Exception:
            refl = None

        if refl is not None:
            for i, cb in enumerate(refl.constantBlocks):
                try:
                    bind = pipe.GetConstantBuffer(stage, i, 0)
                except Exception:
                    continue

                cb_entry = {
                    "index": int(i),
                    "name": str(cb.name),
                    "size": int(cb.byteSize),
                    "resource_id": None,
                    "resource_name": "",
                    "byte_offset": 0,
                }

                rid = getattr(bind, "resourceId", rd.ResourceId.Null())
                if rid != rd.ResourceId.Null():
                    cb_entry["resource_id"] = str(rid)
                    cb_entry["resource_name"] = try_resource_name(controller, rid)
                    cb_entry["byte_offset"] = int(getattr(bind, "byteOffset", 0))

                    if dump_uniform_bytes:
                        try:
                            data = try_get_buffer_data(
                                controller,
                                rid,
                                int(getattr(bind, "byteOffset", 0)),
                                int(cb.byteSize),
                            )
                            if data is not None and len(data) > 0:
                                raw_b64 = base64.b64encode(bytes(data)).decode("ascii")
                                uniform_raw.append(
                                    {
                                        "stage": stage_name,
                                        "cb_index": int(i),
                                        "cb_name": str(cb.name),
                                        "bytes_b64": raw_b64,
                                        "byte_len": int(len(data)),
                                    }
                                )
                                decoded = decode_viewport_uniform(bytes(data))
                                if decoded is not None:
                                    uniform_decoded.append(
                                        {
                                            "stage": stage_name,
                                            "cb_index": int(i),
                                            "cb_name": str(cb.name),
                                            "decoded": decoded,
                                        }
                                    )
                        except Exception:
                            pass

                stage_info["constant_buffers"].append(cb_entry)
        else:
            # Fallback when reflection is unavailable: probe a small range of constant buffer slots.
            for i in range(0, 16):
                try:
                    bind = pipe.GetConstantBuffer(stage, int(i), 0)
                except Exception:
                    continue

                rid = getattr(bind, "resourceId", rd.ResourceId.Null())
                if rid == rd.ResourceId.Null():
                    continue

                byte_offset = int(getattr(bind, "byteOffset", 0))
                byte_size = int(getattr(bind, "byteSize", 0))
                if byte_size <= 0:
                    byte_size = 256

                cb_entry = {
                    "index": int(i),
                    "name": "",
                    "size": int(byte_size),
                    "resource_id": str(rid),
                    "resource_name": try_resource_name(controller, rid),
                    "byte_offset": int(byte_offset),
                }
                stage_info["constant_buffers"].append(cb_entry)

                if dump_uniform_bytes:
                    data = try_get_buffer_data(controller, rid, byte_offset, byte_size)
                    if data is not None and len(data) > 0:
                        raw_b64 = base64.b64encode(bytes(data)).decode("ascii")
                        uniform_raw.append(
                            {
                                "stage": stage_name,
                                "cb_index": int(i),
                                "cb_name": "",
                                "bytes_b64": raw_b64,
                                "byte_len": int(len(data)),
                            }
                        )
                        decoded = decode_viewport_uniform(bytes(data))
                        if decoded is not None:
                            uniform_decoded.append(
                                {
                                    "stage": stage_name,
                                    "cb_index": int(i),
                                    "cb_name": "",
                                    "decoded": decoded,
                                }
                            )

        # Storage buffers tend to show up in read-only/read-write resources.
        try:
            for res in pipe.GetReadOnlyResources(stage, False):
                rid = res.descriptor.resource
                if rid == rd.ResourceId.Null():
                    continue
                stage_info["readonly_resources"].append(
                    {
                        "slot": int(res.access.index),
                        "resource_id": str(rid),
                        "resource_name": try_resource_name(controller, rid),
                    }
                )
        except Exception:
            pass

        try:
            for res in pipe.GetReadWriteResources(stage, False):
                rid = res.descriptor.resource
                if rid == rd.ResourceId.Null():
                    continue
                stage_info["readwrite_resources"].append(
                    {
                        "slot": int(res.access.index),
                        "resource_id": str(rid),
                        "resource_name": try_resource_name(controller, rid),
                    }
                )
        except Exception:
            pass

        stage_dumps.append(stage_info)

    # Buffer dumps (independent of pipeline binding introspection).
    buffer_dump = {
        "uniform_buffer": None,
        "clip_stack_buffer": None,
        "named_buffers": [],
        "selected_uniform_entry": None,
    }
    uniform_stride = int(req.get("uniform_stride", 256))
    uniform_entries = int(req.get("uniform_entries", 32))
    if dump_uniform_bytes and resource_descs is not None:
        try:
            uniform_rid = None
            clip_rid = None
            clip_mask_params_rid = None
            scale_params_rid = None
            color_adjust_params_rid = None
            for desc in resource_descs:
                n = str(getattr(desc, "name", "") or "").lower()
                if uniform_rid is None and "fret quad uniforms buffer" in n:
                    uniform_rid = getattr(desc, "resourceId", None)
                if clip_rid is None and "fret clip stack buffer" in n:
                    clip_rid = getattr(desc, "resourceId", None)
                if clip_mask_params_rid is None and "fret clip-mask params buffer" in n:
                    clip_mask_params_rid = getattr(desc, "resourceId", None)
                if scale_params_rid is None and "fret scale params buffer" in n:
                    scale_params_rid = getattr(desc, "resourceId", None)
                if color_adjust_params_rid is None and "fret color-adjust params buffer" in n:
                    color_adjust_params_rid = getattr(desc, "resourceId", None)

            if uniform_rid is not None:
                size = max(1, uniform_stride) * max(1, uniform_entries)
                data = try_get_buffer_data(controller, uniform_rid, 0, int(size))
                if data is not None and len(data) > 0:
                    raw_b64 = base64.b64encode(bytes(data)).decode("ascii")
                    entries = decode_viewport_uniform_table(bytes(data), uniform_stride, uniform_entries)
                    buffer_dump["uniform_buffer"] = {
                        "resource_id": int(uniform_rid),
                        "resource_name": try_resource_name(controller, uniform_rid),
                        "stride": int(uniform_stride),
                        "entries": entries,
                        "bytes_b64": raw_b64,
                        "byte_len": int(len(data)),
                    }

            if clip_rid is not None:
                data = try_get_buffer_data(controller, clip_rid, 0, 64 * dump_clip_count)
                if data is not None and len(data) > 0:
                    raw_b64 = base64.b64encode(bytes(data)).decode("ascii")
                    buffer_dump["clip_stack_buffer"] = {
                        "resource_id": int(clip_rid),
                        "resource_name": try_resource_name(controller, clip_rid),
                        "entries": decode_clip_rrect_entries(bytes(data), dump_clip_count),
                        "bytes_b64": raw_b64,
                        "byte_len": int(len(data)),
                    }

            if clip_mask_params_rid is not None and "clip mask" in marker_contains:
                data = try_get_buffer_data(controller, clip_mask_params_rid, 0, 16)
                if data is not None and len(data) >= 16:
                    decoded = decode_clip_mask_params(bytes(data))
                    buffer_dump["named_buffers"].append(
                        {
                            "kind": "clip_mask_params",
                            "resource_id": int(clip_mask_params_rid),
                            "resource_name": try_resource_name(controller, clip_mask_params_rid),
                            "decoded": decoded,
                        }
                    )
                    try:
                        if buffer_dump["uniform_buffer"] is not None and decoded is not None:
                            buffer_dump["selected_uniform_entry"] = choose_uniform_entry_for_clip_mask(
                                buffer_dump["uniform_buffer"]["entries"],
                                decoded["dst_size"],
                            )
                    except Exception:
                        pass

            if scale_params_rid is not None and "nearest" in marker_contains:
                scale_params_offset = 0
                try:
                    for stage_info in stage_dumps:
                        for cb in stage_info.get("constant_buffers", []):
                            if "fret scale params buffer" in str(
                                cb.get("resource_name", "") or ""
                            ).lower():
                                scale_params_offset = int(cb.get("byte_offset", 0))
                                raise StopIteration()
                except StopIteration:
                    pass
                except Exception:
                    pass

                data = try_get_buffer_data(controller, scale_params_rid, scale_params_offset, 32)
                if data is not None and len(data) >= 16:
                    buffer_dump["named_buffers"].append(
                        {
                            "kind": "scale_params",
                            "resource_id": int(scale_params_rid),
                            "resource_name": try_resource_name(controller, scale_params_rid),
                            "byte_offset": int(scale_params_offset),
                            "decoded": decode_scale_params(bytes(data)),
                        }
                    )

            if color_adjust_params_rid is not None and "color-adjust" in marker_contains:
                data = try_get_buffer_data(controller, color_adjust_params_rid, 0, 16)
                if data is not None and len(data) >= 16:
                    buffer_dump["named_buffers"].append(
                        {
                            "kind": "color_adjust_params",
                            "resource_id": int(color_adjust_params_rid),
                            "resource_name": try_resource_name(controller, color_adjust_params_rid),
                            "decoded": decode_color_adjust_params(bytes(data)),
                        }
                    )
        except Exception:
            pass

    clip_stack = None
    try:
        # Heuristic: look for a buffer resource named like our clip stack buffer.
        candidates = []
        for stage_info in stage_dumps:
            for r in stage_info.get("readonly_resources", []):
                if "clip stack" in (r.get("resource_name", "").lower()):
                    candidates.append(r)
        if candidates:
            rid = rd.ResourceId(int(candidates[0]["resource_id"]))
            data = controller.GetBufferData(rid, 0, 64 * dump_clip_count)
            clip_stack = {
                "resource_id": int(rid),
                "resource_name": try_resource_name(controller, rid),
                "entries": decode_clip_rrect_entries(bytes(data), dump_clip_count),
            }
    except Exception:
        clip_stack = None

    return {
        "event_id": int(event_id),
        "raster_state": try_viewport_scissor(pipe),
        "outputs": outputs,
        "depth": depth,
        "stages": stage_dumps,
        "uniform_decoded": uniform_decoded,
        "uniform_raw": uniform_raw,
        "clip_stack": clip_stack,
        "resources": resources,
        "buffer_dump": buffer_dump,
    }


def save_outputs_png(controller, event_id: int, out_dir: str, basename: str):
    controller.SetFrameEvent(int(event_id), True)
    pipe = controller.GetPipelineState()

    saved = []
    for i, br in enumerate(pipe.GetOutputTargets()):
        rid = extract_resource_id(br)
        if rid is None or is_null_resource_id(rid):
            continue
        out_path = os.path.join(out_dir, f"{basename}.event{int(event_id)}.rt{i}.png")
        save = rd.TextureSave()
        save.resourceId = rid
        save.destType = rd.FileType.PNG
        save.mip = 0
        # Best effort: support bound descriptors.
        if hasattr(br, "firstMip"):
            try:
                save.mip = int(br.firstMip)
            except Exception:
                pass
        if hasattr(br, "firstSlice"):
            try:
                save.slice = int(br.firstSlice)
            except Exception:
                pass
        result = controller.SaveTexture(save, out_path)
        if result == rd.ResultCode.Succeeded:
            saved.append({"kind": "color", "index": int(i), "output_path": out_path, "resource_id": int(rid)})
    return saved


def main() -> None:
    # qrenderdoc's embedded Python environment is intentionally minimal and may not provide
    # `sys.argv`/argparse reliably. Use fixed request/response file names in the working dir.
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    os.makedirs(req["output_dir"], exist_ok=True)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(req["capture_path"], "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))
        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            matches = []
            iter_actions(None, controller.GetRootActions(), [], matches, req)

            selection = str(req.get("selection", "last"))
            selected = []
            if selection == "all":
                selected = matches
            elif selection == "first":
                if matches:
                    selected = [matches[0]]
            else:
                if matches:
                    selected = [matches[-1]]

            dumps = []
            for m in selected:
                d = dump_event(controller, int(m["event_id"]), req)
                d["match"] = m
                if bool(req.get("save_outputs_png", True)):
                    d["saved_outputs"] = save_outputs_png(
                        controller,
                        int(m["event_id"]),
                        req["output_dir"],
                        req["basename"],
                    )
                dumps.append(d)

            payload = {
                "capture_path": req["capture_path"],
                "matches": matches,
                "selection": selection,
                "dumps": dumps,
            }
            write_response(RESP_PATH, {"ok": True, "result": payload})
        finally:
            try:
                controller.Shutdown()
            except Exception:
                pass
    finally:
        try:
            cap.Shutdown()
        except Exception:
            pass
        rd.ShutdownReplay()


if __name__ == "__main__":
    try:
        main()
    except Exception:
        write_response(RESP_PATH, {"ok": False, "error": traceback.format_exc()})
    raise SystemExit(0)
