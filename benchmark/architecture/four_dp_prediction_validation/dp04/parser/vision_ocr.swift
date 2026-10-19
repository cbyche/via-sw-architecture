import Foundation
import Vision
import ImageIO
let started = DispatchTime.now().uptimeNanoseconds
let url = URL(fileURLWithPath: CommandLine.arguments[1])
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.recognitionLanguages = ["en-US"]
request.usesLanguageCorrection = false
let handler = VNImageRequestHandler(url: url, options: [:])
do {
 try handler.perform([request])
 let observations = request.results ?? []
 let rows: [[String: Any]] = observations.compactMap { observation in
  guard let text = observation.topCandidates(1).first else { return nil }
  let b = observation.boundingBox
  return ["text": text.string, "confidence": text.confidence, "bbox_2d": [Int((b.minX*1000).rounded(.toNearestOrEven)), Int(((1-b.maxY)*1000).rounded(.toNearestOrEven)), Int((b.maxX*1000).rounded(.toNearestOrEven)), Int(((1-b.minY)*1000).rounded(.toNearestOrEven))], "coordinate_space": "qwen_normalized_0_1000"]
 }
 let output: [String: Any] = ["parser": "Apple Vision VNRecognizeTextRequest accurate OCR ONLY", "observations": rows, "duration_ns": DispatchTime.now().uptimeNanoseconds-started, "scope": "Recognized text boxes; no icon detector, captions, target intent, semantic item extents or truth guarantee"]
 let data = try JSONSerialization.data(withJSONObject: output, options: [.sortedKeys])
 print(String(data: data, encoding: .utf8)!)
} catch { fputs("Vision OCR failed: \(error)\n", stderr); exit(1) }
