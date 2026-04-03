#!/usr/bin/env python3
"""
Simple CGI development server for rgit/cgit.

Usage:
    cd /path/to/workdir
    python3 /path/to/examples/serve_with_python.py [--port PORT] [--config CGITRC] [--bind ADDR]

Run from a working directory of your choice. The script will:
  - Create cgi-bin/ and static/ subdirectories there
  - Symlink the cgit binary into cgi-bin/
  - Optionally copy cgit.css from the original cgit source into static/

Visit http://localhost:8080/cgi-bin/cgit/ after starting.
"""

import argparse
import os
import shutil
import sys
from http.server import CGIHTTPRequestHandler, HTTPServer
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Development CGI server for rgit")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--bind", default="127.0.0.1")
    parser.add_argument("--config", help="Path to cgitrc")
    parser.add_argument("--binary", help="Path to cgit binary (default: auto-detect from source tree)")
    parser.add_argument("--cgit-css", help="Path to cgit.css to copy into static/")
    args = parser.parse_args()

    # Find the binary
    if args.binary:
        binary = Path(args.binary).resolve()
    else:
        # Try to find it relative to this script (examples/ -> ../target/release/cgit)
        script_dir = Path(__file__).resolve().parent.parent
        binary = script_dir / "target" / "release" / "cgit"

    if not binary.exists():
        print(f"Binary not found at {binary}", file=sys.stderr)
        print("Run 'cargo build --release' first, or pass --binary.", file=sys.stderr)
        sys.exit(1)

    workdir = Path.cwd()

    # Set up cgi-bin/ with symlink to binary
    cgi_dir = workdir / "cgi-bin"
    cgi_dir.mkdir(exist_ok=True)
    cgi_link = cgi_dir / "cgit"
    if cgi_link.is_symlink() or cgi_link.exists():
        cgi_link.unlink()
    cgi_link.symlink_to(binary)

    # Set up static/ for CSS and assets
    static_dir = workdir / "static"
    static_dir.mkdir(exist_ok=True)

    if args.cgit_css:
        css_src = Path(args.cgit_css)
        if css_src.exists():
            shutil.copy2(css_src, static_dir / "cgit.css")
            print(f"Copied {css_src} -> static/cgit.css")
    else:
        # Try to find cgit.css from the original cgit source tree
        for candidate in [
            binary.parent.parent.parent.parent / "cgit" / "cgit.css",  # rust/../cgit/
            Path("/usr/share/cgit/cgit.css"),
        ]:
            if candidate.exists() and not (static_dir / "cgit.css").exists():
                shutil.copy2(candidate, static_dir / "cgit.css")
                print(f"Copied {candidate} -> static/cgit.css")
                break

    # Set CGIT_CONFIG if provided
    if args.config:
        config = Path(args.config).resolve()
        if config.exists():
            os.environ["CGIT_CONFIG"] = str(config)
            print(f"Using config: {config}")
        else:
            print(f"Error: {config} not found", file=sys.stderr)
            sys.exit(1)

    CGIHTTPRequestHandler.cgi_directories = ["/cgi-bin"]
    server = HTTPServer((args.bind, args.port), CGIHTTPRequestHandler)
    url = f"http://{args.bind}:{args.port}/cgi-bin/cgit/"
    print(f"Serving on {url}")
    print("Press Ctrl+C to stop.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == "__main__":
    main()
