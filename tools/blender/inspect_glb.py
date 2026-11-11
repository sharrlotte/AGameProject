"""Inspect and validate a glTF binary player asset.

Usage:
    python tools/blender/inspect_glb.py game/core/assets/models/player.glb
    python tools/blender/inspect_glb.py game/core/assets/models/player.glb \
        --require-bone WeaponSocket --require-node Foregrip --require-node MuzzleSocket

The script uses only the Python standard library so it can run without Blender.
Exit code is non-zero when a required item is missing or a budget is exceeded.
"""

from __future__ import annotations

import argparse
import json
import struct
import sys
from pathlib import Path

GLB_MAGIC = 0x46546C67
CHUNK_JSON = 0x4E4F534A


def read_gltf_json(path: Path) -> dict:
    data = path.read_bytes()
    magic, _version, length = struct.unpack_from("<III", data, 0)
    if magic != GLB_MAGIC:
        raise ValueError(f"{path} is not a GLB file")
    if length > len(data):
        raise ValueError(f"{path} declares {length} bytes but has {len(data)}")

    offset = 12
    while offset < length:
        chunk_len, chunk_type = struct.unpack_from("<II", data, offset)
        offset += 8
        chunk = data[offset : offset + chunk_len]
        offset += chunk_len
        if chunk_type == CHUNK_JSON:
            return json.loads(chunk.decode("utf-8"))
    raise ValueError(f"{path} has no JSON chunk")


def triangle_count(gltf: dict) -> int:
    total = 0
    accessors = gltf.get("accessors", [])
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            if "indices" in primitive:
                total += accessors[primitive["indices"]]["count"] // 3
            else:
                total += accessors[primitive["attributes"]["POSITION"]]["count"] // 3
    return total


def node_names(gltf: dict) -> list[str]:
    return [node.get("name", f"node_{i}") for i, node in enumerate(gltf.get("nodes", []))]


def animation_names(gltf: dict) -> list[str]:
    return [animation.get("name", f"animation_{i}") for i, animation in enumerate(gltf.get("animations", []))]


def joint_names(gltf: dict) -> list[str]:
    names = node_names(gltf)
    joints: list[str] = []
    for skin in gltf.get("skins", []):
        joints.extend(names[index] for index in skin.get("joints", []))
    return joints


def main() -> int:
    parser = argparse.ArgumentParser(description="Inspect a player GLB")
    parser.add_argument("path", type=Path)
    parser.add_argument("--require-bone", action="append", default=[])
    parser.add_argument("--require-node", action="append", default=[])
    parser.add_argument("--require-clip", action="append", default=[])
    parser.add_argument("--max-triangles", type=int, default=0)
    parser.add_argument("--max-bones", type=int, default=0)
    args = parser.parse_args()

    gltf = read_gltf_json(args.path)
    names = node_names(gltf)
    bones = joint_names(gltf)
    clips = animation_names(gltf)
    triangles = triangle_count(gltf)

    report = {
        "path": str(args.path),
        "nodes": len(names),
        "bones": sorted(bones),
        "bone_count": len(bones),
        "triangles": triangles,
        "clips": sorted(clips),
    }
    print(json.dumps(report, indent=2))

    failures: list[str] = []
    for bone in args.require_bone:
        if bone not in bones:
            failures.append(f"missing bone: {bone}")
    for node in args.require_node:
        if node not in names:
            failures.append(f"missing node: {node}")
    for clip in args.require_clip:
        if clip not in clips:
            failures.append(f"missing clip: {clip}")
    if args.max_triangles and triangles > args.max_triangles:
        failures.append(f"triangle budget exceeded: {triangles} > {args.max_triangles}")
    if args.max_bones and len(bones) > args.max_bones:
        failures.append(f"bone budget exceeded: {len(bones)} > {args.max_bones}")

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
