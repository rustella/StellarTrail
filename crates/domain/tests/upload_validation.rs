use stellartrail_domain::upload::{ImageType, validate_image_upload};

const PNG_1X1: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1, 13, 10,
    45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];
const JPEG_MAGIC: &[u8] = &[0xff, 0xd8, 0xff, 0xdb, 0, 0, 0, 0];
const WEBP_MAGIC: &[u8] = b"RIFF\x08\x00\x00\x00WEBPVP8 ";

#[test]
fn accepts_png_when_extension_content_type_and_magic_match() {
    let validated = validate_image_upload(Some("screen.png"), Some("image/png"), PNG_1X1).unwrap();

    assert_eq!(validated.image_type, ImageType::Png);
    assert_eq!(validated.content_type, "image/png");
    assert_eq!(validated.safe_extension, "png");
    assert_eq!(validated.original_filename, "screen.png");
}

#[test]
fn accepts_jpeg_alias_extensions_when_magic_matches() {
    let jpg = validate_image_upload(Some("photo.jpg"), Some("image/jpeg"), JPEG_MAGIC).unwrap();
    let jpeg = validate_image_upload(Some("photo.jpeg"), Some("image/jpeg"), JPEG_MAGIC).unwrap();

    assert_eq!(jpg.image_type, ImageType::Jpeg);
    assert_eq!(jpg.safe_extension, "jpg");
    assert_eq!(jpeg.image_type, ImageType::Jpeg);
    assert_eq!(jpeg.safe_extension, "jpg");
}

#[test]
fn accepts_webp_when_extension_content_type_and_magic_match() {
    let validated =
        validate_image_upload(Some("screen.webp"), Some("image/webp"), WEBP_MAGIC).unwrap();

    assert_eq!(validated.image_type, ImageType::Webp);
    assert_eq!(validated.content_type, "image/webp");
    assert_eq!(validated.safe_extension, "webp");
}

#[test]
fn rejects_octet_stream_when_content_type_is_declared() {
    let error = validate_image_upload(
        Some("screen.png"),
        Some("application/octet-stream"),
        PNG_1X1,
    )
    .unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}

#[test]
fn rejects_extension_magic_mismatch() {
    let error = validate_image_upload(Some("screen.jpg"), Some("image/jpeg"), PNG_1X1).unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}

#[test]
fn rejects_declared_content_type_magic_mismatch() {
    let error = validate_image_upload(Some("screen.png"), Some("image/jpeg"), PNG_1X1).unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}

#[test]
fn rejects_html_payload_with_image_extension() {
    let error = validate_image_upload(
        Some("payload.png"),
        Some("image/png"),
        b"<script>alert('xss')</script>",
    )
    .unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}

#[test]
fn rejects_svg_even_when_declared_as_image() {
    let error = validate_image_upload(
        Some("vector.svg"),
        Some("image/svg+xml"),
        br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#,
    )
    .unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}

#[test]
fn rejects_unknown_extension_even_with_valid_magic() {
    let error =
        validate_image_upload(Some("payload.php"), Some("image/jpeg"), JPEG_MAGIC).unwrap_err();

    assert!(error.fields.iter().any(|field| field.field == "file"));
}
