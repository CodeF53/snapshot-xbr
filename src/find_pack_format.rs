use cafebabe::{attributes::AttributeData, constant_pool::LiteralConstant, parse_class};
use regex::Regex;
use std::{fs, io::Read};
use zip::ZipArchive;

pub fn find_pack_format() -> Result<i32, Box<dyn std::error::Error>> {
	let mappings_str = fs::read_to_string("./tmp/client.txt")?;

	let class_name = Regex::new("SharedConstants -> ([a-z]+):")?
		.captures(&mappings_str)
		.expect("couldn't find SharedConstants class")
		.get(1)
		.unwrap()
		.as_str();
	let field_name = Regex::new("RESOURCE_PACK_FORMAT(?:_MAJOR)? -> ([a-z]+)")?
		.captures(&mappings_str)
		.expect("couldn't find RESOURCE_PACK_FORMAT_MAJOR field")
		.get(1)
		.unwrap()
		.as_str();

	let mut client_zip = ZipArchive::new(fs::File::open("./tmp/client.jar")?)?;
	let mut class_zip = client_zip.by_name(&format!("{class_name}.class"))?;

	let mut class_bytes = Vec::new();
	class_zip.read_to_end(&mut class_bytes)?;
	let class = parse_class(&class_bytes)?;

	Ok(class
		.fields
		.iter()
		.find(|field| field.name == field_name)
		.and_then(|field| {
			field.attributes.iter().find_map(|attr| match &attr.data {
				AttributeData::ConstantValue(LiteralConstant::Integer(val)) => Some(*val),
				_ => None,
			})
		})
		.expect("Constant value for field not found."))
}
