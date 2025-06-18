# Knottycam

This camera filter is inspired by the Infinite Loop game where you would turn separate arcs and lines into a continuous path. Similarly, the filter creates continuous shapes based on camera input.

> ⚠️ This is a concept, and a way for me to learn Rust by building fun stuff. You may use this code in any way you like, but please don't launch it on your machine before reviewing the code.

## Example

![Example](example.mp4)

---

The current implementation just outputs into a new window, not actually creating a virtual camera device. If you want to use it, you will have to write your own implementation or use OBS-like software.

When mapping Luminance to layers of the output, I have intentionally used values up to 100, even though it should cover Luminance up to 255. I don't have dark background, so I opt in for this cutout effect while being isolated from the background.
