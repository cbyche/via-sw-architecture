using System.Security.Cryptography;
using System.Text.Json;
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

if (args.Length != 2) throw new ArgumentException("Expected PPTX path and JSON report path.");
string source = Path.GetFullPath(args[0]);
string hash = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(source))).ToLowerInvariant();
using var document = PresentationDocument.Open(source, false);
var validator = new OpenXmlValidator(FileFormatVersions.Office2021) { MaxNumberOfErrors = 10000 };
var errors = validator.Validate(document).Select(e => new {
    id = e.Id, type = e.ErrorType.ToString(), description = e.Description,
    part = e.Part?.Uri.ToString(), xpath = e.Path?.XPath
}).ToArray();
int slides = document.PresentationPart?.Presentation.SlideIdList?.ChildElements.Count ?? 0;
var report = new {
    sha256 = hash, os = System.Runtime.InteropServices.RuntimeInformation.OSDescription,
    sdk = "DocumentFormat.OpenXml 3.3.0", format = "Office2021",
    opened = true, slide_count = slides, schema_error_count = errors.Length, errors,
    native_powerpoint_verified = false,
    claim_boundary = "Microsoft SDK package/schema validation on Windows. This does not establish PowerPoint rendering or GIF playback."
};
File.WriteAllText(args[1], JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
Console.WriteLine($"Opened {slides} slides; schema errors: {errors.Length}; SHA256: {hash}");
if (errors.Length != 0) Environment.ExitCode = 1;
