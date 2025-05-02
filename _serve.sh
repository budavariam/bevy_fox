#!/bin/sh
cd out || exit 1
python3 -m http.server 8000 --bind 127.0.0.1
