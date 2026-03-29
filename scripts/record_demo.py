#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import time
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "assets" / "demo.mp4"
WIDTH = 1920
HEIGHT = 1080
FPS = 1
SERVER_URL = "http://127.0.0.1:7878/web/index.html"

SCENES = [
    {
        "title": "from-zero-to-systems",
        "subtitle": (
            "Learn Rust through codebase evolution\n"
            "Each crate depends on earlier ones\n"
            "At arc boundaries you refactor, benchmark,\n"
            "and maintain what you built"
        ),
        "is_title_card": True,
        "duration": 4,
    },
    {
        "annotation": (
            "Three-column workspace. Briefing and guide on the left,\n"
            "code editor in the centre, concepts and checkpoints on the right."
        ),
        "action": "screenshot",
        "duration": 7,
    },
    {
        "annotation": (
            "Tests drive each challenge. Write the minimum to make them pass.\n"
            "The runner gives immediate feedback."
        ),
        "action": "run_tests",
        "duration": 7,
    },
    {
        "annotation": (
            "Navigate between challenges. Later crates import earlier ones\n"
            "as library dependencies, building a real workspace."
        ),
        "action": "select_challenge_06",
        "duration": 7,
    },
    {
        "annotation": (
            "Six arcs from probability to machine learning.\n"
            "At each arc boundary, evolution milestones force you\n"
            "to maintain what you built."
        ),
        "action": "select_challenge_13",
        "duration": 7,
    },
]


def get_font(size: int):
    font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-Bold.ttf",
    ]
    for fp in font_paths:
        if Path(fp).exists():
            return ImageFont.truetype(fp, size)
    return ImageFont.load_default()


def get_font_regular(size: int):
    font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-Regular.ttf",
    ]
    for fp in font_paths:
        if Path(fp).exists():
            return ImageFont.truetype(fp, size)
    return ImageFont.load_default()


def create_title_card(title: str, subtitle: str, frame_path: Path) -> None:
    img = Image.new("RGB", (WIDTH, HEIGHT), color=(15, 23, 42))
    draw = ImageDraw.Draw(img)

    title_font = get_font(64)
    sub_font = get_font_regular(30)

    title_bbox = draw.textbbox((0, 0), title, font=title_font)
    title_w = title_bbox[2] - title_bbox[0]
    draw.text(
        ((WIDTH - title_w) // 2, HEIGHT // 2 - 140),
        title,
        fill=(96, 165, 250),
        font=title_font,
    )

    for i, line in enumerate(subtitle.split("\n")):
        line_bbox = draw.textbbox((0, 0), line, font=sub_font)
        line_w = line_bbox[2] - line_bbox[0]
        draw.text(
            ((WIDTH - line_w) // 2, HEIGHT // 2 - 10 + i * 46),
            line,
            fill=(203, 213, 225),
            font=sub_font,
        )

    img.save(frame_path)


def add_annotation(screenshot_path: Path, annotation: str, output_path: Path) -> None:
    img = Image.open(screenshot_path)
    img = img.resize((WIDTH, HEIGHT), Image.LANCZOS)

    overlay = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)

    bar_height = 100
    draw.rectangle(
        ((0, HEIGHT - bar_height), (WIDTH, HEIGHT)),
        fill=(15, 23, 42, 230),
    )

    font = get_font_regular(26)
    lines = annotation.split("\n")
    y = HEIGHT - bar_height + 15
    for line in lines:
        line_bbox = draw.textbbox((0, 0), line, font=font)
        line_w = line_bbox[2] - line_bbox[0]
        draw.text(
            ((WIDTH - line_w) // 2, y),
            line,
            fill=(226, 232, 240, 255),
            font=font,
        )
        y += 36

    img = img.convert("RGBA")
    img = Image.alpha_composite(img, overlay)
    img.convert("RGB").save(output_path)


def click_challenge(page, index: int) -> None:
    buttons = page.locator(".challenge-list button")
    count = buttons.count()
    if index < count:
        buttons.nth(index).click()
        time.sleep(2)


def run_demo() -> None:
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    frames_dir = ROOT / "demo_frames"
    frames_dir.mkdir(exist_ok=True)

    for f in frames_dir.glob("*.png"):
        f.unlink()

    frame_num = 0

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": WIDTH, "height": HEIGHT})

        print("Connecting to fzts server...")
        for attempt in range(20):
            try:
                page.goto(SERVER_URL, wait_until="networkidle", timeout=15000)
                page.wait_for_selector(".workspace-grid", timeout=10000)
                break
            except Exception:
                if attempt == 19:
                    raise RuntimeError(
                        f"Server not responding at {SERVER_URL}. "
                        "Start it with: cargo run -p play web"
                    )
                time.sleep(2)

        print("Connected.")
        time.sleep(2)

        for scene in SCENES:
            label = scene.get("action", scene.get("title", "card"))
            print(f"  Frame {frame_num}: {label}")

            if scene.get("is_title_card"):
                for _ in range(scene["duration"] * FPS):
                    frame_path = frames_dir / f"frame_{frame_num:04d}.png"
                    create_title_card(scene["title"], scene["subtitle"], frame_path)
                    frame_num += 1
                continue

            action = scene.get("action", "screenshot")

            if action == "run_tests":
                btn = page.locator(".primary-button").first
                if btn.is_visible():
                    btn.click()
                    time.sleep(4)

            elif action == "select_challenge_06":
                click_challenge(page, 5)

            elif action == "select_challenge_13":
                click_challenge(page, 12)

            time.sleep(1)

            screenshot_path = frames_dir / f"raw_{frame_num:04d}.png"
            page.screenshot(path=str(screenshot_path), full_page=False)

            annotation = scene.get("annotation", "")
            for _ in range(scene["duration"] * FPS):
                frame_path = frames_dir / f"frame_{frame_num:04d}.png"
                if annotation:
                    add_annotation(screenshot_path, annotation, frame_path)
                else:
                    img = Image.open(screenshot_path)
                    img = img.resize((WIDTH, HEIGHT), Image.LANCZOS)
                    img.save(frame_path)
                frame_num += 1

            screenshot_path.unlink(missing_ok=True)

        browser.close()

    print(f"Generated {frame_num} frames. Encoding video...")

    cmd = [
        "ffmpeg", "-y",
        "-framerate", str(FPS),
        "-i", str(frames_dir / "frame_%04d.png"),
        "-c:v", "libx264",
        "-pix_fmt", "yuv420p",
        "-r", "30",
        "-preset", "medium",
        "-crf", "23",
        str(OUTPUT),
    ]
    subprocess.run(cmd, check=True)

    for f in frames_dir.glob("*.png"):
        f.unlink()
    frames_dir.rmdir()

    print(f"Done. Video saved to {OUTPUT}")


if __name__ == "__main__":
    run_demo()
