#!/usr/bin/env python3
"""
Simple CGI development server for rgit/cgit.

Usage:
    ./serve.py [--port PORT] [--config CGITRC] [--bind ADDR]

Defaults to port 8080, looks for cgitrc in the current directory.
Visit http://localhost:8080/cgit/ after starting.
"""

import argparse
import os
import sys
from http.server import CGIHTTPRequestHandler, HTTPServer
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Development CGI server for rgit")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--bind", default="127.0.0.1")
    parser.add_argument("--config", default=None, help="Path to cgitrc (default: ./cgitrc)")
    args = parser.parse_args()

    script_dir = Path(__file__).resolve().parent
    binary = script_dir / "target" / "release" / "cgit"
    if not binary.exists():
        print(f"Binary not found at {binary}", file=sys.stderr)
        print("Run 'cargo build --release' first.", file=sys.stderr)
        sys.exit(1)

    # Set up cgi-bin directory with symlink to the binary
    cgi_dir = script_dir / "cgi-bin"
    cgi_dir.mkdir(exist_ok=True)
    cgi_link = cgi_dir / "cgit"
    if cgi_link.is_symlink() or cgi_link.exists():
        cgi_link.unlink()
    cgi_link.symlink_to(binary)

    # Resolve cgitrc path
    config = Path(args.config) if args.config else script_dir / "cgitrc"
    if config.exists():
        os.environ["CGIT_CONFIG"] = str(config.resolve())
        print(f"Using config: {config.resolve()}")
    else:
        print(f"Warning: {config} not found, cgit will use defaults", file=sys.stderr)

    os.chdir(script_dir)

    CGIHTTPRequestHandler.cgi_directories = ["/cgi-bin"]
    server = HTTPServer((args.bind, args.port), CGIHTTPRequestHandler)
    print(f"Serving on http://{args.bind}:{args.port}/cgi-bin/cgit/")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == "__main__":
    main()
