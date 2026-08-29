#!/usr/bin/env python3
"""Validate the bit-context Codex skill package and OpenAI metadata."""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml


ALLOWED_FRONTMATTER_KEYS = {
    "name",
    "description",
    "license",
    "allowed-tools",
    "metadata",
}


def fail(message: str) -> None:
    raise ValueError(message)


def load_skill_frontmatter(skill_file: Path) -> tuple[dict[str, object], str]:
    content = skill_file.read_text(encoding="utf-8")
    match = re.match(r"^---\n(.*?)\n---\n", content, re.DOTALL)
    if match is None:
        fail("SKILL.md must start with YAML frontmatter")
    data = yaml.safe_load(match.group(1))
    if not isinstance(data, dict):
        fail("SKILL.md frontmatter must be a mapping")
    return data, content[match.end() :]


def validate_frontmatter(frontmatter: dict[str, object]) -> None:
    unexpected = set(frontmatter) - ALLOWED_FRONTMATTER_KEYS
    if unexpected:
        fail(f"unexpected SKILL.md frontmatter keys: {sorted(unexpected)}")

    name = frontmatter.get("name")
    if name != "bit-context":
        fail("SKILL.md name must be 'bit-context'")

    description = frontmatter.get("description")
    if not isinstance(description, str) or not description.strip():
        fail("SKILL.md description must be a non-empty string")
    if len(description) > 1024 or "<" in description or ">" in description:
        fail("SKILL.md description violates length or character constraints")
    for required_phrase in ("verified", "deterministic", "Do not use"):
        if required_phrase not in description:
            fail(f"SKILL.md description must contain '{required_phrase}'")


def validate_body(body: str) -> None:
    required_phrases = (
        "command -v bitctx",
        "Do not install it automatically",
        "Never invent",
        "external authorization",
        "plaintext JSON",
        "--session",
    )
    for phrase in required_phrases:
        if phrase not in body:
            fail(f"SKILL.md is missing required guidance: {phrase}")


def validate_openai_yaml(metadata_file: Path) -> None:
    raw = metadata_file.read_text(encoding="utf-8")
    data = yaml.safe_load(raw)
    if not isinstance(data, dict):
        fail("agents/openai.yaml must be a mapping")

    interface = data.get("interface")
    if not isinstance(interface, dict):
        fail("agents/openai.yaml requires an interface mapping")
    if interface.get("display_name") != "Bit Context":
        fail("display_name must be 'Bit Context'")

    short_description = interface.get("short_description")
    if not isinstance(short_description, str) or not 25 <= len(short_description) <= 64:
        fail("short_description must contain 25-64 characters")

    default_prompt = interface.get("default_prompt")
    if not isinstance(default_prompt, str) or "$bit-context" not in default_prompt:
        fail("default_prompt must explicitly mention $bit-context")

    policy = data.get("policy")
    if not isinstance(policy, dict) or policy.get("allow_implicit_invocation") is not True:
        fail("implicit invocation must be explicitly enabled")

    for line_number, line in enumerate(raw.splitlines(), start=1):
        match = re.match(r"^\s+[a-z_]+:\s+(.+)$", line)
        if match is None:
            continue
        value = match.group(1).strip()
        if value not in {"true", "false"} and not (
            value.startswith('"') and value.endswith('"')
        ):
            fail(f"string value must be quoted at agents/openai.yaml:{line_number}")


def validate_package(skill_dir: Path) -> None:
    required_files = (
        "SKILL.md",
        "agents/openai.yaml",
        "bitctx_skill.sh",
        "example_schema.json",
    )
    for relative_path in required_files:
        if not (skill_dir / relative_path).is_file():
            fail(f"missing skill package file: {relative_path}")

    frontmatter, body = load_skill_frontmatter(skill_dir / "SKILL.md")
    validate_frontmatter(frontmatter)
    validate_body(body)
    validate_openai_yaml(skill_dir / "agents/openai.yaml")


def main() -> int:
    skill_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "skills/bit-context")
    try:
        validate_package(skill_dir)
    except (OSError, ValueError, yaml.YAMLError) as error:
        print(f"Skill validation failed: {error}", file=sys.stderr)
        return 1
    print("Skill validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
