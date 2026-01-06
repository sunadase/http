from http.server import HTTPServer, BaseHTTPRequestHandler
import sys

class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        print("--- Headers received ---")
        # headers is an email.message.Message object. 
        # In Python 3, it is case-insensitive dictionary-like, BUT keys() preserves original case?
        # Actually http.server parses headers.
        # Let's inspect raw headers if possible or iterate keys.
        # Python's http.server stores headers in self.headers.
        # We want to see the casing.
        # str(self.headers) prints them.
        print(self.headers)
        print("------------------------")
        
        casing_correct = False
        if "X-Custom-Header" in str(self.headers):
             casing_correct = True
        
        self.send_response(200)
        self.end_headers()
        
        if casing_correct:
             print("VERIFICATION SUCCESS: X-Custom-Header found")
        else:
             print("VERIFICATION FAILURE: X-Custom-Header NOT found in raw output")

server = HTTPServer(('localhost', 8000), Handler)
print("Server listening on 8000...")
server.handle_request()
