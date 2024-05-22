#! /usr/bin/env python3

from http.server import BaseHTTPRequestHandler, HTTPServer
import time

# Define the hostname and port to listen on
hostname = "localhost"
port = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        import argparse

        args = argparse.ArgumentParser(description="A script to serve the necessary files for the flake builder.")
        # args.add_argument("--enable-lb", help="add --lb bootstrap arg", default="false")
        # boolean
        args.add_argument("--enable-lb", help="add --lb bootstrap arg", action="store_true")
        args = args.parse_args()
        if self.path == "/turso_token":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"")
        elif self.path == "/file_encryption_key":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f")
        elif self.path == "/template_id":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f")
        elif self.path == "/flake_url":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"/nix/store/yaak6mprs2w5b2vrf5dzq2lwd76mpry0-test-flake#hello-flakery")
        # deployment_id
        elif self.path == "/deployment_id":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"00f00f")
        # bootstrap_args
        elif self.path == "/bootstrap_args" and args.enable_lb:
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"--lb")
        elif self.path == "/bootstrap_args":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"")
        else:
            self.send_response(404)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"Path not found")

if __name__ == "__main__":

    webServer = HTTPServer((hostname, port), MyServer)
    print(f"Server started http://{hostname}:{port}")

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")
