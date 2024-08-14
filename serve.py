#!/usr/bin/env python3

from http.server import BaseHTTPRequestHandler, HTTPServer
import json

# Define the hostname and port to listen on
hostname = "localhost"
port = 8080

class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/api/deployments/lb-config-ng":
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.end_headers()
            response = {
                "http": {
                    "routers": {
                        "finer-snail-230f97.flakery.xyz": {"service": "230f97a2-8e84-4d9b-8246-11caf8e4507a"},
                    },
                    "services": {
                        "230f97a2-8e84-4d9b-8246-11caf8e4507a": {"servers": [{"url": "http://machine2:8080"}]},
                    },
                },
            }
            self.wfile.write(json.dumps(response).encode())
