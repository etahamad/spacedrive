use std::fmt::Display;

use exif::Tag;

use crate::ExifReader;

#[derive(Default, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Dimensions {
	pub width: i32,
	pub height: i32,
}

impl Dimensions {
	#[must_use]
	/// Creates a new width and height container
	///
	/// # Examples
	///
	/// ```
	/// use sd_media_data::Dimensions;
	///
	/// Dimensions::new(1920, 1080);
	/// ```
	pub const fn new(width: i32, height: i32) -> Self {
		Self { width, height }
	}

	#[must_use]
	pub fn from_reader(reader: &ExifReader) -> Self {
		Self {
			width: reader
				.get_tag(Tag::PixelXDimension)
				.unwrap_or_else(|| reader.get_tag(Tag::XResolution).unwrap_or_default()),
			height: reader
				.get_tag(Tag::PixelYDimension)
				.unwrap_or_else(|| reader.get_tag(Tag::YResolution).unwrap_or_default()),
		}
	}
}

impl Display for Dimensions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}x{}", self.width, self.height))
	}
}
