extern crate nom;

use std::convert::TryInto;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::combinator::{map, peek};
use nom::error::VerboseError;
use nom::multi::count;
use nom::number::complete::{be_u16, le_u16, le_u32, le_u8};
use nom::sequence::{preceded, tuple};
use nom::IResult;


mod cp437;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Header {
	pub vendor: [char; 3],
	pub product: u16,
	pub serial: u32,
	pub week: u8,
	pub year: u8, // Starting at year 1990
	pub version: u8,
	pub revision: u8,
}

fn parse_vendor(v: u16) -> [char; 3] {
	let mask: u8 = 0x1F; // Each letter is 5 bits
	let i0 = b'A' - 1; // 0x01 = A
	[
		(((v >> 10) as u8 & mask) + i0) as char,
		(((v >> 5) as u8 & mask) + i0) as char,
		((v as u8 & mask) + i0) as char,
	]
}
fn parse_header(input: &[u8]) -> IResult<&[u8], Header, VerboseError<&[u8]>> {
    // Define the parsing sequence
    map(
        tuple((
            tag(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]), // Match the fixed tag
            be_u16,                                                 // Big-endian u16 for vendor
            le_u16,                                                 // Little-endian u16 for product
            le_u32,                                                 // Little-endian u32 for serial
            le_u8,                                                  // Little-endian u8 for week
            le_u8,                                                  // Little-endian u8 for year
            le_u8,                                                  // Little-endian u8 for version
            le_u8,                                                  // Little-endian u8 for revision
        )),
        |(_tag, vendor, product, serial, week, year, version, revision)| Header {
            vendor: parse_vendor(vendor),
            product,
            serial,
            week,
            year,
            version,
            revision,
        },
    )(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Display {
	pub video_input: u8,
	pub width: u8, // cm
	pub height: u8, // cm
	pub gamma: u8, // datavalue = (gamma*100)-100 (range 1.00–3.54)
	pub features: u8,
}
fn parse_display(input: &[u8]) -> IResult<&[u8], Display, VerboseError<&[u8]>> {
    map(
        tuple((
            le_u8, // Video input field
            le_u8, // Width
            le_u8, // Height
            le_u8, // Gamma
            le_u8, // Features
        )),
        |(video_input, width, height, gamma, features)| Display {
            video_input,
            width,
            height,
            gamma,
            features,
        },
    )(input)
}

fn parse_chromaticity(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    let (remaining, _) = take(10usize)(input)?; // Consume 10 bytes
    Ok((remaining, ())) // Return unit type
}

fn parse_established_timing(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    let (remaining, _) = take(3usize)(input)?; // Consume 10 bytes
    Ok((remaining, ())) // Return unit type
}

fn parse_standard_timing(input: &[u8]) -> IResult<&[u8], (), VerboseError<&[u8]>> {
    let (remaining, _) = take(16usize)(input)?; // Consume 10 bytes
    Ok((remaining, ())) // Return unit type
}

// Function to parse descriptor text
fn parse_descriptor_text(input: &[u8]) -> IResult<&[u8], String> {
    // Parse exactly 13 bytes, filter and convert them
    map(
        map(take(13_usize), |b: &[u8]| {
            b.iter()
                .filter(|&&c| c != 0x0A) // Filter out newline characters
                .map(|&b| cp437::forward(b)) // Translate using CP437
                .collect::<String>()
        }),
        |s| s.trim().to_string(), // Trim the resulting string
    )(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct DetailedTiming {
	/// Pixel clock in kHz.
	pub pixel_clock: u32,
	pub horizontal_active_pixels: u16,
	pub horizontal_blanking_pixels: u16,
	pub vertical_active_lines: u16,
	pub vertical_blanking_lines: u16,
	pub horizontal_front_porch: u16,
	pub horizontal_sync_width: u16,
	pub vertical_front_porch: u16,
	pub vertical_sync_width: u16,
	/// Horizontal size in millimeters
	pub horizontal_size: u16,
	/// Vertical size in millimeters
	pub vertical_size: u16,
	/// Border pixels on one side of screen (i.e. total number is twice this)
	pub horizontal_border_pixels: u8,
	/// Border pixels on one side of screen (i.e. total number is twice this)
	pub vertical_border_pixels: u8,
	pub features: u8, /* TODO add enums etc. */
}

fn parse_detailed_timing(input: &[u8]) -> IResult<&[u8], DetailedTiming> {
    map(
        tuple((
            le_u16, // pixel_clock_10khz
            le_u8,  // horizontal_active_lo
            le_u8,  // horizontal_blanking_lo
            le_u8,  // horizontal_px_hi
            le_u8,  // vertical_active_lo
            le_u8,  // vertical_blanking_lo
            le_u8,  // vertical_px_hi
            le_u8,  // horizontal_front_porch_lo
            le_u8,  // horizontal_sync_width_lo
            le_u8,  // vertical_lo
            le_u8,  // porch_sync_hi
            le_u8,  // horizontal_size_lo
            le_u8,  // vertical_size_lo
            le_u8,  // size_hi
            le_u8,  // horizontal_border
            le_u8,  // vertical_border
            le_u8,  // features
        )),
        |(
            pixel_clock_10khz,
            horizontal_active_lo,
            horizontal_blanking_lo,
            horizontal_px_hi,
            vertical_active_lo,
            vertical_blanking_lo,
            vertical_px_hi,
            horizontal_front_porch_lo,
            horizontal_sync_width_lo,
            vertical_lo,
            porch_sync_hi,
            horizontal_size_lo,
            vertical_size_lo,
            size_hi,
            horizontal_border,
            vertical_border,
            features,
        )| DetailedTiming {
            pixel_clock: pixel_clock_10khz as u32 * 10,
            horizontal_active_pixels: (horizontal_active_lo as u16)
                | (((horizontal_px_hi >> 4) as u16) << 8),
            horizontal_blanking_pixels: (horizontal_blanking_lo as u16)
                | (((horizontal_px_hi & 0xf) as u16) << 8),
            vertical_active_lines: (vertical_active_lo as u16)
                | (((vertical_px_hi >> 4) as u16) << 8),
            vertical_blanking_lines: (vertical_blanking_lo as u16)
                | (((vertical_px_hi & 0xf) as u16) << 8),
            horizontal_front_porch: (horizontal_front_porch_lo as u16)
                | (((porch_sync_hi >> 6) as u16) << 8),
            horizontal_sync_width: (horizontal_sync_width_lo as u16)
                | ((((porch_sync_hi >> 4) & 0x3) as u16) << 8),
            vertical_front_porch: ((vertical_lo >> 4) as u16)
                | ((((porch_sync_hi >> 2) & 0x3) as u16) << 8),
            vertical_sync_width: ((vertical_lo & 0xf) as u16)
                | (((porch_sync_hi & 0x3) as u16) << 8),
            horizontal_size: (horizontal_size_lo as u16) | (((size_hi >> 4) as u16) << 8),
            vertical_size: (vertical_size_lo as u16) | (((size_hi & 0xf) as u16) << 8),
            horizontal_border_pixels: horizontal_border,
            vertical_border_pixels: vertical_border,
            features,
        },
    )(input)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Descriptor {
	DetailedTiming(DetailedTiming),
	SerialNumber(String),
	UnspecifiedText(String),
	RangeLimits, // TODO
	ProductName(String),
	WhitePoint, // TODO
	StandardTiming, // TODO
	ColorManagement,
	TimingCodes,
	EstablishedTimings,
	Dummy,
	Unknown([u8; 13]),
}


// Parser for Descriptor
fn parse_descriptor(input: &[u8]) -> IResult<&[u8], Descriptor> {
    let (input, descriptor_type) = peek(le_u16)(input)?;

    match descriptor_type {
        0 => {
            // Descriptor block starts
            let (input, _) = take(3_usize)(input)?;
            let (input, descriptor) = preceded(
                le_u8,
                alt((
                    map(preceded(le_u8, parse_descriptor_text), Descriptor::SerialNumber),
                    map(preceded(le_u8, parse_descriptor_text), Descriptor::UnspecifiedText),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::RangeLimits),
                    map(preceded(le_u8, parse_descriptor_text), Descriptor::ProductName),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::WhitePoint),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::StandardTiming),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::ColorManagement),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::TimingCodes),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::EstablishedTimings),
                    map(preceded(le_u8, take(13_usize)), |_| Descriptor::Dummy),
                    map(
                        preceded(le_u8, count(le_u8, 13)),
                        |data: Vec<u8>| Descriptor::Unknown(data.try_into().unwrap()),
                    ),
                )),
            )(input)?;
            Ok((input, descriptor))
        }
        _ => {
            // Detailed timing block
            map(parse_detailed_timing, Descriptor::DetailedTiming)(input)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EDID {
	pub header: Header,
	pub display: Display,
	chromaticity: (), // TODO
	established_timing: (), // TODO
	standard_timing: (), // TODO
	pub descriptors: Vec<Descriptor>,
}

pub fn parse_edid(input: &[u8]) -> IResult<&[u8], EDID, VerboseError<&[u8]>> {
    let (input, header) = parse_header(input)?;
    let (input, display) = parse_display(input)?;
    let (input, chromaticity) = parse_chromaticity(input)?;
    let (input, established_timing) = parse_established_timing(input)?;
    let (input, standard_timing) = parse_standard_timing(input)?;
    let (input, descriptors) = count(parse_descriptor, 4)(input).unwrap();
    let (input, _) = take(1usize)(input)?; // Consume the extensions byte
    let (input, _) = take(1usize)(input)?; // Consume the checksum byte

    Ok((
        input,
        EDID {
            header,
            display,
            chromaticity,
            established_timing,
            standard_timing,
            descriptors,
        },
    ))
}

pub fn parse(data: &[u8]) -> nom::IResult<&[u8], EDID, VerboseError<&[u8]>> {
	parse_edid(data)
}


//#[cfg(test)]
//mod tests {
//	use super::*;
//
//	fn test(d: &[u8], expected: &EDID) {
//		match parse(d) {
//			nom::IResult::Done(remaining, parsed) => {
//				assert_eq!(remaining.len(), 0);
//				assert_eq!(&parsed, expected);
//			},
//			nom::IResult::Error(err) => {
//				panic!("{}", err);
//			},
//			nom::IResult::Incomplete(_) => {
//				panic!("Incomplete");
//			},
//		}
//	}
//
//	#[test]
//	fn test_card0_vga_1() {
//		let d = include_bytes!("../testdata/card0-VGA-1");
//
//		let expected = EDID{
//			header: Header{
//				vendor: ['S', 'A', 'M'],
//				product: 596,
//				serial: 1146106418,
//				week: 27,
//				year: 17,
//				version: 1,
//				revision: 3,
//			},
//			display: Display{
//				video_input: 14,
//				width: 47,
//				height: 30,
//				gamma: 120,
//				features: 42,
//			},
//			chromaticity: (),
//			established_timing: (),
//			standard_timing: (),
//			descriptors: vec!(
//				Descriptor::DetailedTiming(DetailedTiming {
//					pixel_clock: 146250,
//					horizontal_active_pixels: 1680,
//					horizontal_blanking_pixels: 560,
//					vertical_active_lines: 1050,
//					vertical_blanking_lines: 39,
//					horizontal_front_porch: 104,
//					horizontal_sync_width: 176,
//					vertical_front_porch: 3,
//					vertical_sync_width: 6,
//					horizontal_size: 474,
//					vertical_size: 296,
//					horizontal_border_pixels: 0,
//					vertical_border_pixels: 0,
//					features: 28
//				}),
//				Descriptor::RangeLimits,
//				Descriptor::ProductName("SyncMaster".to_string()),
//				Descriptor::SerialNumber("HS3P701105".to_string()),
//			),
//		};
//
//		test(d, &expected);
//	}
//
//	#[test]
//	fn test_card0_edp_1() {
//		let d = include_bytes!("../testdata/card0-eDP-1");
//
//		let expected = EDID{
//			header: Header{
//				vendor: ['S', 'H', 'P'],
//				product: 5193,
//				serial: 0,
//				week: 32,
//				year: 25,
//				version: 1,
//				revision: 4,
//			},
//			display: Display{
//				video_input: 165,
//				width: 29,
//				height: 17,
//				gamma: 120,
//				features: 14,
//			},
//			chromaticity: (),
//			established_timing: (),
//			standard_timing: (),
//			descriptors: vec!(
//				Descriptor::DetailedTiming(DetailedTiming {
//					pixel_clock: 138500,
//					horizontal_active_pixels: 1920,
//					horizontal_blanking_pixels: 160,
//					vertical_active_lines: 1080,
//					vertical_blanking_lines: 31,
//					horizontal_front_porch: 48,
//					horizontal_sync_width: 32,
//					vertical_front_porch: 3,
//					vertical_sync_width: 5,
//					horizontal_size: 294,
//					vertical_size: 165,
//					horizontal_border_pixels: 0,
//					vertical_border_pixels: 0,
//					features: 24,
//				}),
//				Descriptor::Dummy,
//				Descriptor::UnspecifiedText("DJCP6ÇLQ133M1".to_string()),
//				Descriptor::Unknown([2, 65, 3, 40, 0, 18, 0, 0, 11, 1, 10, 32, 32]),
//			),
//		};
//
//		test(d, &expected);
//	}
//}
