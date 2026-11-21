// Regenerate assets/macos/Chord.icns from assets/chord.svg:
//   swift packaging/macos-icon.swift assets/chord.svg /tmp/Chord.iconset && iconutil -c icns /tmp/Chord.iconset -o assets/macos/Chord.icns
import AppKit

let args = CommandLine.arguments
guard let img = NSImage(contentsOfFile: args[1]) else { fatalError("cannot load \(args[1])") }
let out = args[2]
try FileManager.default.createDirectory(atPath: out, withIntermediateDirectories: true)
// Apple's icon grid puts an 824 px body on a 1024 px canvas; the SVG's body spans 8..248 of 256.
for (name, px) in [("16x16", 16), ("16x16@2x", 32), ("32x32", 32), ("32x32@2x", 64), ("128x128", 128), ("128x128@2x", 256), ("256x256", 256), ("256x256@2x", 512), ("512x512", 512), ("512x512@2x", 1024)] {
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: px, pixelsHigh: px, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    let size = Double(px)
    let full = size * 824 / 1024 * 256 / 240
    let origin = (size - full) / 2
    img.draw(in: NSRect(x: origin, y: origin, width: full, height: full))
    NSGraphicsContext.restoreGraphicsState()
    try rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: "\(out)/icon_\(name).png"))
}
