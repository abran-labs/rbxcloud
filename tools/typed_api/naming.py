"""Deterministic names for generated typed API modules."""
import keyword
import re

RUST_RESERVED = {"as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "box", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"}


def snake(value: str) -> str:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    value = re.sub(r"[^A-Za-z0-9]+", "_", value).strip("_").lower()
    if not value:
        raise ValueError("empty Rust name")
    if value[0].isdigit():
        value = "n_" + value
    if value in RUST_RESERVED or keyword.iskeyword(value):
        value += "_"
    return value


def pascal(value: str) -> str:
    return "".join(piece[:1].upper() + piece[1:] for piece in snake(value).split("_") if piece)


def kebab(value: str) -> str:
    return snake(value).removesuffix("_").replace("_", "-")
