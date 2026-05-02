#!/usr/bin/env python3
"""Capture e-Gov law page screenshot + DOM text for OCR eval fixtures.

See docs/eval_data_egov.md for setup and usage.
"""

from __future__ import annotations

import argparse
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable, Sequence
from urllib.parse import urlparse

# e-Gov: `#MainProvision` is the current-body text (matches typical OCR target);
# `article.law` includes 目次・歴史的表記など広い範囲。
DEFAULT_CONTENT_SELECTORS: Sequence[str] = (
    "#MainProvision",
    "article.law",
    "main.main-content",
    "main",
    "[role='main']",
    "article",
    "#law-body",
    ".law-body",
    "body",
)


def sanitize_stem(raw: str) -> str:
    t = raw.strip()
    if not t:
        return "fixture"
    s = re.sub(r"[^0-9A-Za-z_.-]+", "_", t)
    s = re.sub(r"_+", "_", s).strip("_")
    return s or "fixture"


def derive_fixture_stem(url: str, override: str | None) -> str:
    if override is not None:
        return sanitize_stem(override)
    parsed = urlparse(url.strip())
    m = re.search(r"/law/([^/?#]+)", parsed.path or "")
    if m:
        return sanitize_stem(f"law_{m.group(1)}")
    path = (parsed.path or "").strip("/").replace("/", "_") or "page"
    host = (parsed.netloc or "host").replace(":", "_")
    return sanitize_stem(f"{host}_{path}")[:120]


def postprocess_truth_text(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    lines = text.split("\n")
    out: list[str] = []
    blank_run = False
    for line in lines:
        normalized = " ".join(line.split())
        if not normalized:
            blank_run = True
            continue
        if blank_run and out:
            out.append("")
        out.append(normalized)
        blank_run = False
    return "\n".join(out).strip()


def _first_matching_selector(page: object, selectors: Iterable[str]) -> tuple[str, object]:
    last_err: Exception | None = None
    for sel in selectors:
        try:
            handle = page.locator(sel).first
            handle.wait_for(state="visible", timeout=15_000)
            return sel, handle
        except Exception as e:
            last_err = e
            continue
    raise RuntimeError(f"no content selector matched: {list(selectors)}") from last_err


def capture_fixture(
    *,
    url: str,
    images_dir: Path,
    truth_dir: Path,
    stem: str,
    screenshot_target: str,
    full_page: bool,
    viewport_width: int,
    viewport_height: int,
    goto_timeout_ms: int,
    content_selectors: Sequence[str],
    metadata_path: Path | None,
) -> None:
    from playwright.sync_api import sync_playwright

    images_dir.mkdir(parents=True, exist_ok=True)
    truth_dir.mkdir(parents=True, exist_ok=True)
    png_path = images_dir / f"{stem}.png"
    txt_path = truth_dir / f"{stem}.txt"

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        try:
            page = browser.new_page(viewport={"width": viewport_width, "height": viewport_height})
            page.goto(url, wait_until="domcontentloaded", timeout=goto_timeout_ms)
            page.wait_for_load_state("networkidle", timeout=goto_timeout_ms)
            sel_used, loc = _first_matching_selector(page, content_selectors)
            text = loc.inner_text(timeout=15_000)
            if screenshot_target == "content":
                loc.screenshot(path=str(png_path))
            elif screenshot_target == "page":
                page.screenshot(path=str(png_path), full_page=full_page)
            else:
                raise ValueError(f"unknown screenshot_target: {screenshot_target!r}")
            txt_path.write_text(postprocess_truth_text(text) + "\n", encoding="utf-8")
            if metadata_path is not None:
                meta_lines = [
                    f"source_url: {url}",
                    f"captured_at_utc: {datetime.now(timezone.utc).isoformat()}",
                    f"content_selector: {sel_used}",
                    f"screenshot_target: {screenshot_target}",
                    f"full_page: {full_page}",
                    f"viewport: {viewport_width}x{viewport_height}",
                    "",
                ]
                metadata_path.parent.mkdir(parents=True, exist_ok=True)
                metadata_path.write_text("\n".join(meta_lines), encoding="utf-8")
        finally:
            browser.close()


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Capture e-Gov law page for OCR eval fixtures.")
    p.add_argument("--url", required=True, help="Law page URL (laws.e-gov.go.jp/law/...)")
    p.add_argument(
        "--images-dir",
        type=Path,
        default=Path("tests/fixtures/eval/images"),
        help="Output PNG directory",
    )
    p.add_argument(
        "--truth-dir",
        type=Path,
        default=Path("tests/fixtures/eval/truth"),
        help="Output truth txt directory",
    )
    p.add_argument("--stem", default=None, help="Override output stem (filename without extension)")
    p.add_argument(
        "--screenshot-target",
        choices=("content", "page"),
        default="content",
        help="content: PNG = matched element (aligned with truth). page: viewport/full-page capture.",
    )
    p.add_argument(
        "--full-page",
        action="store_true",
        help="With screenshot-target=page only: capture full scrollable page",
    )
    p.add_argument("--viewport-width", type=int, default=1280)
    p.add_argument("--viewport-height", type=int, default=720)
    p.add_argument("--goto-timeout-ms", type=int, default=120_000)
    p.add_argument(
        "--metadata",
        type=Path,
        default=None,
        help="Optional sidecar file (e.g. tmp/egov_law_129AC0000000089.txt) with URL and timestamp",
    )
    p.add_argument(
        "--selector",
        action="append",
        default=[],
        help="Extra content selector (tried before defaults). Repeatable.",
    )
    p.add_argument(
        "--dry-run",
        action="store_true",
        help="Print stem and paths only (no browser)",
    )
    return p.parse_args()


def main() -> None:
    args = parse_args()
    stem = derive_fixture_stem(args.url, args.stem)
    png_path = args.images_dir / f"{stem}.png"
    txt_path = args.truth_dir / f"{stem}.txt"
    if args.dry_run:
        print(f"stem={stem}")
        print(f"png={png_path}")
        print(f"truth={txt_path}")
        return

    selectors: list[str] = []
    selectors.extend(args.selector)
    selectors.extend(DEFAULT_CONTENT_SELECTORS)

    capture_fixture(
        url=args.url,
        images_dir=args.images_dir,
        truth_dir=args.truth_dir,
        stem=stem,
        screenshot_target=args.screenshot_target,
        full_page=args.full_page,
        viewport_width=args.viewport_width,
        viewport_height=args.viewport_height,
        goto_timeout_ms=args.goto_timeout_ms,
        content_selectors=selectors,
        metadata_path=args.metadata,
    )
    print(f"wrote {png_path}")
    print(f"wrote {txt_path}")
    if args.metadata is not None:
        print(f"wrote {args.metadata}")


if __name__ == "__main__":
    main()
