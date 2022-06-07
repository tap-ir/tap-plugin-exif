//! Exif plugin take a data attribute from a node and extract exif info if compatible with this plugin 

use std::str;

use tap::attribute::Attributes;
use tap::config_schema;
use tap::plugin;
use tap::plugin::{PluginInfo, PluginInstance, PluginConfig, PluginArgument, PluginResult, PluginEnvironment};
use tap::vfile::{VFile};
use tap::tree::{TreeNodeId, VecTreeNodeIdSchema};
use tap::error::RustructError;

use schemars::{JsonSchema};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, NaiveDateTime, Utc};
use inflector::Inflector;

plugin!("exif", "Metadata",  "Extract EXIF info from file", Exif, Arguments);

#[derive(Default)]
pub struct Exif 
{
}

#[derive(Debug, Serialize, Deserialize,JsonSchema)]
pub struct Arguments
{
  #[schemars(with = "VecTreeNodeIdSchema")]
  files : Vec<TreeNodeId>,
}

#[derive(Debug, Serialize, Deserialize,Default)]
pub struct Results
{
}

impl Exif
{
  pub fn add_field_as_attributes(&self, attributes : &mut Attributes, tag : exif::Tag, field : &exif::Field)
  {
    let name = tag.to_string().to_snake_case();
    match &field.value
    {
      exif::Value::Byte(_) | exif::Value::Short(_) | exif::Value::Long(_) =>  if let Some(int) = field.value.get_uint(0)
      {
        attributes.add_attribute(name, int, None);
      },
      exif::Value::Rational(rational) =>   attributes.add_attribute(name, rational[0].to_f64(), None),
      exif::Value::SRational(rational) => attributes.add_attribute(name, rational[0].to_f64(), None),
      exif::Value::Ascii(vec) => match tag 
      {
        exif::Tag::DateTime | exif::Tag::DateTimeOriginal | exif::Tag::DateTimeDigitized => if !vec.is_empty()
        {
          if let Ok(str_datetime) = str::from_utf8(&vec[0])
          {
            if let Ok(datetime) = NaiveDateTime::parse_from_str(str_datetime, "%Y:%m:%d %H:%M:%S")
            {
              attributes.add_attribute(name, DateTime::<Utc>::from_utc(datetime, Utc), None); 
            }
          }
        },
        _ => attributes.add_attribute(name, field.display_value().to_string(), None),
      },
      _ => attributes.add_attribute(name, field.display_value().to_string(), None),
    }
  }

  pub fn to_attributes(&self, file : Box<dyn VFile>) -> Option<Attributes>
  {
    if let Ok(reader) = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file))
    {
      let tag_list = [exif::Tag::ImageWidth, exif::Tag::ImageLength, exif::Tag::XResolution,
                      exif::Tag::YResolution, exif::Tag::Make, exif::Tag::Model,
                      exif::Tag::Software, exif::Tag::Artist, exif::Tag::Copyright,
                      exif::Tag::ImageDescription, exif::Tag::DateTime, exif::Tag::DateTimeOriginal,
                      exif::Tag::DateTimeDigitized,]; //add gps 

      let mut primary = Attributes::new();
      let mut thumbnail = Attributes::new();

      for &tag in tag_list.iter()
      {
        if let Some(field) = reader.get_field(tag, exif::In::PRIMARY)
        {
          self.add_field_as_attributes(&mut primary, tag, field);
        }
        
        if let Some(field) = reader.get_field(tag, exif::In::THUMBNAIL)
        {
          self.add_field_as_attributes(&mut thumbnail, tag, field);
        }
      }
     
      let mut attributes = Attributes::new();
      attributes.add_attribute("primary", primary, None);
      if thumbnail.count() > 0
      {
        attributes.add_attribute("thumbnail", thumbnail, None);
      }
      return Some(attributes)
    }
    None
  }

  fn run(&mut self, args : Arguments, env : PluginEnvironment) -> anyhow::Result<Results>
  {
    for file in args.files
    {
      let file_node = env.tree.get_node_from_id(file).ok_or(RustructError::ArgumentNotFound("file"))?;
      let data = file_node.value().get_value("data").ok_or(RustructError::ValueNotFound("data"))?;
      let data_builder = data.try_as_vfile_builder().ok_or(RustructError::ValueTypeMismatch)?;

      if let Ok(file) = data_builder.open()
      {
         match self.to_attributes(file)
         {
           Some(attributes) => file_node.value().add_attribute(self.name(), attributes, None),
           None =>  file_node.value().add_attribute(self.name(), None, None),
         }
      }
    }

    Ok(Results{})
  }
}
