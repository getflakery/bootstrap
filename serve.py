#! /usr/bin/env python3

from http.server import BaseHTTPRequestHandler, HTTPServer
import time

# Define the hostname and port to listen on
hostname = "localhost"
port = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
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
