#!/usr/bin/env python3
"""Scan a Lightning invoice QR code via webcam and print the BOLT11 string."""
import sys
import cv2
from pyzbar.pyzbar import decode

cap = cv2.VideoCapture(0)
if not cap.isOpened():
    print("Error: cannot open webcam", file=sys.stderr)
    sys.exit(1)

print("Point webcam at a Lightning invoice QR code. Press 'q' to quit.", file=sys.stderr)

while True:
    ret, frame = cap.read()
    if not ret:
        break
    for obj in decode(frame):
        data = obj.data.decode("utf-8").strip()
        # Strip lightning: URI scheme if present
        if data.lower().startswith("lightning:"):
            data = data[len("lightning:"):]
        if data.lower().startswith("lnbc"):
            cap.release()
            cv2.destroyAllWindows()
            print(data)
            sys.exit(0)
    cv2.imshow("Scan QR - press q to quit", frame)
    if cv2.waitKey(1) & 0xFF == ord("q"):
        break

cap.release()
cv2.destroyAllWindows()
print("No invoice scanned", file=sys.stderr)
sys.exit(1)
