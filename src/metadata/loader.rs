// Copyright (C) 2017 1aim GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::str;
use std::io::{BufRead};

use xml::reader::Reader;
use xml::events::{self, Event};
use xml::events::attributes::Attribute;

use error::{self, Result};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Defaults {
	format:     Format,
	descriptor: Descriptor,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Metadata {
	pub general:          Option<Descriptor>,
	pub fixed_line:       Option<Descriptor>,
	pub mobile:           Option<Descriptor>,
	pub toll_free:        Option<Descriptor>,
	pub premium_rate:     Option<Descriptor>,
	pub shared_cost:      Option<Descriptor>,
	pub personal:         Option<Descriptor>,
	pub voip:             Option<Descriptor>,
	pub pager:            Option<Descriptor>,
	pub uan:              Option<Descriptor>,
	pub emergency:        Option<Descriptor>,
	pub voicemail:        Option<Descriptor>,
	pub short_code:       Option<Descriptor>,
	pub standard_rate:    Option<Descriptor>,
	pub carrier:          Option<Descriptor>,
	pub no_international: Option<Descriptor>,

	pub id:           Option<String>,
	pub country_code: Option<u16>,

	pub international_prefix:           Option<String>,
	pub preferred_international_prefix: Option<String>,
	pub national_prefix:                Option<String>,
	pub preferred_extension_prefix:     Option<String>,

	pub national_prefix_for_parsing: Option<String>,
	pub national_prefix_transform_rule: Option<String>,

	pub format:               Vec<Format>,
	pub international_format: Vec<Format>,

	pub main_country_for_code: bool,
	pub leading_digits: Option<String>,
	pub mobile_number_portable: bool,

	pub defaults: Defaults,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Format {
	pub pattern: Option<String>,
	pub format: Option<String>,
	pub leading_digits: Vec<String>,
	pub national_prefix: Option<String>,
	pub national_prefix_formatting_rule: Option<String>,
	pub national_prefix_optional_when_formatting: bool,
	pub domestic_carrier: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Descriptor {
	pub national_number: Option<String>,
	pub possible_number: Option<String>,
	pub possible_length: Vec<u16>,
	pub possible_local_length: Vec<u16>,
	pub example: Option<String>,
}

pub fn load<R: BufRead>(reader: R) -> Result<Vec<Metadata>> {
	metadata(&mut Reader::from_reader(reader))
}

pub fn metadata<'a, R: BufRead>(reader: &mut Reader<R>) -> Result<Vec<Metadata>> {
	let mut buffer = Vec::new();
	let mut result = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Eof =>
				return Ok(result),

			Event::Start(ref e) => {
				match e.name() {
					b"phoneNumberMetadata" =>
						continue,

					b"territories" =>
						result.extend(territories(reader)?),

					name =>
						ignore(reader, name)?,
				}
			}

			Event::End(ref e) if e.name() != b"phoneNumberMetadata" =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			_ => ()
		}
	}
}

fn territories<'a, R: BufRead>(reader: &mut Reader<R>) -> Result<Vec<Metadata>> {
	let mut buffer = Vec::new();
	let mut result = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					b"territory" =>
						result.push(territory(reader, e)?),

					name =>
						ignore(reader, e.name())?,
				}
			}

			Event::End(ref e) if e.name() == b"territories" =>
				return Ok(result),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn territory<'a, R: BufRead>(reader: &mut Reader<R>, e: &events::BytesStart<'a>) -> Result<Metadata> {
	let mut buffer = Vec::new();
	let mut meta   = Metadata::default();

	for attr in e.attributes() {
		let Attribute { key, value } = attr?;

		match (str::from_utf8(key)?, str::from_utf8(value)?) {
			("id", value) =>
				meta.id = Some(value.into()),

			("countryCode", value) =>
				meta.country_code = Some(value.parse()?),

			("internationalPrefix", value) =>
				meta.international_prefix = Some(value.into()),

			("preferredInternationalPrefix", value) =>
				meta.preferred_international_prefix = Some(value.into()),

			("nationalPrefix", value) =>
				meta.national_prefix = Some(value.into()),

			("preferredExtnPrefix", value) =>
				meta.preferred_extension_prefix = Some(value.into()),

			("nationalPrefixForParsing", value) =>
				meta.national_prefix_for_parsing = Some(value.into()),

			("nationalPrefixTransformRule", value) =>
				meta.national_prefix_transform_rule = Some(value.into()),

			("mainCountryForCode", value) =>
				meta.main_country_for_code = value.parse()?,

			("leadingDigits", value) =>
				meta.leading_digits = Some(value.into()),

			("mobileNumberPortableRegion", value) =>
				meta.mobile_number_portable = value.parse()?,

			("nationalPrefixFormattingRule", value) =>
				meta.defaults.format.national_prefix_formatting_rule = Some(value.into()),

			("nationalPrefixOptionalWhenFormatting", value) =>
				meta.defaults.format.national_prefix_optional_when_formatting = value.parse()?,

			("carrierCodeFormattingRule", value) =>
				meta.defaults.format.domestic_carrier = Some(value.into()),

			(name, value) =>
				return Err(error::Metadata::UnhandledAttribute {
					phase: "format".into(),
					name:  name.into(),
					value: value.into()
				}.into())
		}
	}

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"references" =>
						ignore(reader, name)?,

					name @ b"generalDesc" =>
						meta.general = Some(descriptor(reader, &meta, name)?),

					name @ b"fixedLine" =>
						meta.fixed_line = Some(descriptor(reader, &meta, name)?),

					name @ b"mobile" =>
						meta.mobile = Some(descriptor(reader, &meta, name)?),

					name @ b"tollFree" =>
						meta.toll_free = Some(descriptor(reader, &meta, name)?),

					name @ b"premiumRate" =>
						meta.premium_rate = Some(descriptor(reader, &meta, name)?),

					name @ b"sharedCost" =>
						meta.shared_cost = Some(descriptor(reader, &meta, name)?),

					name @ b"personalNumber" =>
						meta.personal = Some(descriptor(reader, &meta, name)?),

					name @ b"voip" =>
						meta.voip = Some(descriptor(reader, &meta, name)?),

					name @ b"pager" =>
						meta.pager = Some(descriptor(reader, &meta, name)?),

					name @ b"uan" =>
						meta.uan = Some(descriptor(reader, &meta, name)?),

					name @ b"emergency" =>
						meta.emergency = Some(descriptor(reader, &meta, name)?),

					name @ b"voicemail" =>
						meta.voicemail = Some(descriptor(reader, &meta, name)?),

					name @ b"noInternationalDialling" =>
						meta.no_international = Some(descriptor(reader, &meta, name)?),

					name @ b"availableFormats" => {
						let (national, international) = formats(reader, &meta, name)?;

						meta.format               = national;
						meta.international_format = international;
					}

					name @ b"areaCodeOptional" =>
						ignore(reader, name)?,

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "territory".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == b"territory" =>
				return Ok(meta),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn descriptor<R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8]) -> Result<Descriptor> {
	let mut buffer     = Vec::new();
	let mut descriptor = meta.defaults.descriptor.clone();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"nationalNumberPattern" =>
						descriptor.national_number = Some(text(reader, name)?),

					name @ b"exampleNumber" =>
						descriptor.example = Some(text(reader, name)?),

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "descriptor".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok(descriptor),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn formats<R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8]) -> Result<(Vec<Format>, Vec<Format>)> {
	let mut buffer        = Vec::new();
	let mut national      = Vec::new();
	let mut international = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"numberFormat" => {
						let (natl, intl) = format(reader, meta, name, e)?;

						national.push(natl);

						if let Some(intl) = intl {
							international.push(intl);
						}
					}

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "formats".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok((national, international)),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn format<'a, R: BufRead>(reader: &mut Reader<R>, meta: &Metadata, name: &[u8], e: &events::BytesStart<'a>) -> Result<(Format, Option<Format>)> {
	let mut buffer = Vec::new();

	let mut format        = meta.defaults.format.clone();
	let mut international = None;

	for attr in e.attributes() {
		let Attribute { key, value } = attr?;

		match (str::from_utf8(key)?, str::from_utf8(value)?) {
			("pattern", value) =>
				format.pattern = Some(value.into()),

			("nationalPrefixFormattingRule", value) =>
				format.national_prefix_formatting_rule = Some(value.into()),

			("nationalPrefixOptionalWhenFormatting", value) =>
				format.national_prefix_optional_when_formatting = value.parse()?,

			("carrierCodeFormattingRule", value) =>
				format.domestic_carrier = Some(value.into()),

			(name, value) =>
				return Err(error::Metadata::UnhandledAttribute {
					phase: "format".into(),
					name:  name.into(),
					value: value.into()
				}.into())
		}
	}

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name @ b"leadingDigits" =>
						format.leading_digits.push(text(reader, name)?),

					name @ b"format" => {
						let text = text(reader, name)?;

						format.format = Some(text.clone());
						international = Some(text);
					}

					name @ b"intlFormat" => {
						let text = text(reader, name)?;

						if text == "NA" {
							international = None;
						}
						else {
							international = Some(text);
						}
					}

					name =>
						return Err(error::Metadata::UnhandledElement {
							phase: "format".into(),
							name:  str::from_utf8(name)?.into(),
						}.into())
				}
			}

			Event::End(ref e) if e.name() == name => {
				let international = international.map(|v| {
					let mut format = format.clone();
					format.format = Some(v);
					format
				});

				return Ok((format, international));
			}

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn ignore<'a, R: BufRead>(reader: &mut Reader<R>, name: &[u8]) -> Result<()> {
	let mut buffer = Vec::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Start(ref e) => {
				match e.name() {
					name =>
						ignore(reader, name)?,
				}
			}

			Event::End(ref e) if e.name() == name =>
				return Ok(()),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}

fn text<'a, R: BufRead>(reader: &mut Reader<R>, name: &[u8]) -> Result<String> {
	let mut buffer = Vec::new();
	let mut result = String::new();

	loop {
		match reader.read_event(&mut buffer)? {
			Event::Text(ref e) =>
				result.push_str(str::from_utf8(e)?),

			Event::End(ref e) if e.name() == name =>
				return Ok(result),

			Event::End(ref e) =>
				return Err(error::Metadata::MismatchedTag(
					str::from_utf8(e.name())?.into()).into()),

			Event::Eof =>
				return Err(error::Metadata::UnexpectedEof.into()),

			_ =>
				()
		}
	}
}