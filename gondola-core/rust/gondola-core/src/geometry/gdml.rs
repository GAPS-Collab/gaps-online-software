//! Streaming parser for the subset of GDML used by the 
//! GAPS geometry.
//!
//! GDML requires every element to be defined before it is 
//! referenced (define -> materials -> solids -> structure), 
//! so all references are resolved while reading.
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use std::io::BufRead;

use glam::DVec3;
use quick_xml::events::{
  BytesStart,
  Event,
};
use quick_xml::Reader;

/// Parse a GDML file
pub fn parse_gdml_file(path : &Path) -> Result<Geometry, GeometryError> {
  let file = File::open(path)?;
  parse(Reader::from_reader(BufReader::new(file)))
}

/// Parse GDML from a string
pub fn parse_gdml_str(gdml : &str) -> Result<Geometry, GeometryError> {
  parse(Reader::from_str(gdml))
}

//------------------------------------------------------------

/// Error without position, the position is added
/// by the main loop
type Res<T> = Result<T, String>;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Section {
  None,
  Define,
  Materials,
  Solids,
  Structure,
  Setup,
}

/// The GDML element which is currently being read and
/// can have children
enum Open {
  Material  { name : String, density : Option<f64> },
  Tessellated(TessellatedBuilder),
  Boolean(BooleanBuilder),
  Volume(VolumeBuilder),
  Physvol(VolumeBuilder, PhysvolBuilder),
}

struct TessellatedBuilder {
  name      : String,
  lunit     : f64,
  vertices  : Vec<DVec3>,
  triangles : Vec<[u32; 3]>,
  /// define name -> index in vertices
  index     : HashMap<String, u32>,
}

struct BooleanBuilder {
  name     : String,
  op       : BooleanOp,
  first    : Option<SolidId>,
  second   : Option<SolidId>,
  position : DVec3,
  angles   : DVec3,
}

struct VolumeBuilder {
  name      : String,
  material  : Option<MaterialId>,
  solid     : Option<SolidId>,
  daughters : Vec<Placement>,
  auxiliary : Vec<(String, String)>,
}

struct PhysvolBuilder {
  name       : String,
  copynumber : i64,
  volume     : Option<VolumeId>,
  position   : DVec3,
  angles     : DVec3,
}

#[derive(Default)]
struct Parser {
  /// define section, in mm and rad
  positions    : HashMap<String, DVec3>,
  rotations    : HashMap<String, DVec3>,
  material_ids : HashMap<String, MaterialId>,
  solid_ids    : HashMap<String, SolidId>,
  volume_ids   : HashMap<String, VolumeId>,
  materials    : Vec<Material>,
  solids       : Vec<Solid>,
  volumes      : Vec<Volume>,
  world        : Option<VolumeId>,
}

fn parse<R: BufRead>(mut reader : Reader<R>) -> Result<Geometry, GeometryError> {
  let mut parser  = Parser::default();
  let mut section = Section::None;
  let mut open    : Option<Open> = None;
  let mut buf     = Vec::new();
  loop {
    let pos   = reader.buffer_position();
    let event = reader.read_event_into(&mut buf)?;
    let invalid = |msg : String| GeometryError::Gdml { pos, msg };
    match event {
      Event::Start(e) => {
        parser.start(&e, &mut section, &mut open).map_err(invalid)?;
      }
      Event::Empty(e) => {
        parser.start(&e, &mut section, &mut open).map_err(invalid)?;
        parser.end(e.name().as_ref(), &mut section, &mut open).map_err(invalid)?;
      }
      Event::End(e) => {
        parser.end(e.name().as_ref(), &mut section, &mut open).map_err(invalid)?;
      }
      Event::Eof => break,
      _ => (),
    }
    buf.clear();
  }
  let world = parser.world.ok_or_else(|| GeometryError::Gdml {
    pos : reader.buffer_position(),
    msg : String::from("No world volume defined in <setup>"),
  })?;
  Geometry::new(parser.materials, parser.solids, parser.volumes, world)
}

impl Parser {
  fn start(&mut self, e : &BytesStart, section : &mut Section, open : &mut Option<Open>) -> Res<()> {
    let tag   = String::from_utf8_lossy(e.name().as_ref()).into_owned();
    let attrs = Attributes::new(e)?;
    match (*section, tag.as_str()) {
      (_, "gdml")                     => (),
      (Section::None, "define")       => *section = Section::Define,
      (Section::None, "materials")    => *section = Section::Materials,
      (Section::None, "solids")       => *section = Section::Solids,
      (Section::None, "structure")    => *section = Section::Structure,
      (Section::None, "setup")        => *section = Section::Setup,
      (Section::Define, _)            => self.define(&tag, &attrs)?,
      (Section::Materials, _)         => self.material_start(&tag, &attrs, open)?,
      (Section::Solids, _)            => self.solid_start(&tag, &attrs, open)?,
      (Section::Structure, _)         => self.structure_start(&tag, &attrs, open)?,
      (Section::Setup, "world")       => {
        self.world = Some(lookup(&self.volume_ids, attrs.get("ref")?, "volume")?);
      }
      (section, _) => return Err(format!("Unexpected element <{tag}> in section {section:?}")),
    }
    Ok(())
  }

  fn end(&mut self, tag : &[u8], section : &mut Section, open : &mut Option<Open>) -> Res<()> {
    match tag {
      b"define" | b"materials" | b"solids" | b"structure" | b"setup" => {
        if open.is_some() {
          return Err(format!("Section {:?} closed while an element is still open", section));
        }
        *section = Section::None;
      }
      b"material" => {
        if let Some(Open::Material {name, density}) = open.take() {
          let density = density.ok_or_else(|| format!("Material {name} has no density <D>"))?;
          self.material_ids.insert(name.clone(), self.materials.len());
          self.materials.push(Material { name, density });
        }
      }
      b"tessellated" => {
        if let Some(Open::Tessellated(t)) = open.take() {
          self.push_solid(t.name, Shape::Tessellated {
            vertices  : t.vertices,
            triangles : t.triangles,
          });
        }
      }
      b"union" | b"intersection" | b"subtraction" => {
        if let Some(Open::Boolean(b)) = open.take() {
          let first  = b.first.ok_or_else(|| format!("Boolean solid {} has no <first>", b.name))?;
          let second = b.second.ok_or_else(|| format!("Boolean solid {} has no <second>", b.name))?;
          self.push_solid(b.name, Shape::Boolean {
            op               : b.op,
            first,
            second,
            second_transform : Transform::from_gdml(b.position, b.angles),
          });
        }
      }
      b"physvol" => {
        if let Some(Open::Physvol(mut v, p)) = open.take() {
          let volume = p.volume.ok_or_else(|| format!("physvol {} has no <volumeref>", p.name))?;
          v.daughters.push(Placement {
            name       : p.name,
            copynumber : p.copynumber,
            volume,
            transform  : Transform::from_gdml(p.position, p.angles),
          });
          *open = Some(Open::Volume(v));
        }
      }
      b"volume" if *section == Section::Structure => {
        if let Some(Open::Volume(v)) = open.take() {
          let material = v.material.ok_or_else(|| format!("Volume {} has no <materialref>", v.name))?;
          let solid    = v.solid.ok_or_else(|| format!("Volume {} has no <solidref>", v.name))?;
          self.volume_ids.insert(v.name.clone(), self.volumes.len());
          self.volumes.push(Volume {
            name      : v.name,
            material,
            solid,
            daughters : v.daughters,
            auxiliary : v.auxiliary,
          });
        }
      }
      _ => (),
    }
    Ok(())
  }

  fn define(&mut self, tag : &str, attrs : &Attributes) -> Res<()> {
    match tag {
      "position" => {
        let name = attrs.get("name")?.to_string();
        let pos  = attrs.vec3(length_unit(attrs.get_or("unit", "mm"))?)?;
        self.positions.insert(name, pos);
      }
      "rotation" => {
        let name = attrs.get("name")?.to_string();
        let rot  = attrs.vec3(angle_unit(attrs.get_or("unit", "rad"))?)?;
        self.rotations.insert(name, rot);
      }
      // constants, variables, matrices, ... would require
      // an expression evaluator
      _ => return Err(format!("Unsupported element <{tag}> in <define>")),
    }
    Ok(())
  }

  fn material_start(&mut self, tag : &str, attrs : &Attributes, open : &mut Option<Open>) -> Res<()> {
    match (tag, open.as_mut()) {
      ("material", None) => {
        *open = Some(Open::Material { name : attrs.get("name")?.to_string(), density : None });
      }
      ("D", Some(Open::Material { density, .. })) => {
        *density = Some(attrs.f64("value")? * density_unit(attrs.get_or("unit", "g/cm3"))?);
      }
      // elements and isotopes as well as the composition
      // and other properties of the materials are not needed
      // for the column density
      _ => (),
    }
    Ok(())
  }

  fn solid_start(&mut self, tag : &str, attrs : &Attributes, open : &mut Option<Open>) -> Res<()> {
    match open.as_mut() {
      None => match tag {
        "tessellated" => {
          *open = Some(Open::Tessellated(TessellatedBuilder {
            name      : attrs.get("name")?.to_string(),
            lunit     : length_unit(attrs.get_or("lunit", "mm"))?,
            vertices  : Vec::new(),
            triangles : Vec::new(),
            index     : HashMap::new(),
          }));
        }
        "box" => {
          let lunit = length_unit(attrs.get_or("lunit", "mm"))?;
          // GDML gives the full lengths
          let half_lengths = 0.5*attrs.vec3(lunit)?;
          self.push_solid(attrs.get("name")?.to_string(), Shape::Box { half_lengths });
        }
        "tube" => {
          let lunit = length_unit(attrs.get_or("lunit", "mm"))?;
          let aunit = angle_unit(attrs.get_or("aunit", "rad"))?;
          let shape = Shape::Tube {
            rmin      : attrs.f64_or("rmin", 0.0)? * lunit,
            rmax      : attrs.f64("rmax")? * lunit,
            // GDML gives the full length
            half_z    : 0.5*attrs.f64("z")? * lunit,
            start_phi : attrs.f64_or("startphi", 0.0)? * aunit,
            delta_phi : attrs.f64("deltaphi")? * aunit,
          };
          self.push_solid(attrs.get("name")?.to_string(), shape);
        }
        "union" | "intersection" | "subtraction" => {
          let op = match tag {
            "union"        => BooleanOp::Union,
            "intersection" => BooleanOp::Intersection,
            _              => BooleanOp::Subtraction,
          };
          *open = Some(Open::Boolean(BooleanBuilder {
            name     : attrs.get("name")?.to_string(),
            op,
            first    : None,
            second   : None,
            position : DVec3::ZERO,
            angles   : DVec3::ZERO,
          }));
        }
        _ => return Err(format!("Unsupported solid <{tag}>")),
      },
      Some(Open::Tessellated(t)) => {
        let vertex = |t : &mut TessellatedBuilder, positions : &HashMap<String, DVec3>, key : &str| -> Res<u32> {
          let name = attrs.get(key)?;
          if let Some(k) = t.index.get(name) {
            return Ok(*k);
          }
          let pos = *positions.get(name).ok_or_else(|| format!("Unknown vertex {name}"))?;
          let k   = t.vertices.len() as u32;
          // as in Geant4, the lunit of the solid is applied
          // on top of the unit of the position
          t.vertices.push(pos*t.lunit);
          t.index.insert(name.to_string(), k);
          Ok(k)
        };
        if attrs.get_or("type", "ABSOLUTE") != "ABSOLUTE" {
          return Err(format!("Facets of type {} are not supported", attrs.get_or("type", "")));
        }
        match tag {
          "triangular" => {
            let a = vertex(t, &self.positions, "vertex1")?;
            let b = vertex(t, &self.positions, "vertex2")?;
            let c = vertex(t, &self.positions, "vertex3")?;
            t.triangles.push([a, b, c]);
          }
          "quadrangular" => {
            let a = vertex(t, &self.positions, "vertex1")?;
            let b = vertex(t, &self.positions, "vertex2")?;
            let c = vertex(t, &self.positions, "vertex3")?;
            let d = vertex(t, &self.positions, "vertex4")?;
            t.triangles.push([a, b, c]);
            t.triangles.push([a, c, d]);
          }
          _ => return Err(format!("Unsupported element <{tag}> in <tessellated>")),
        }
      }
      Some(Open::Boolean(b)) => match tag {
        "first"       => b.first    = Some(lookup(&self.solid_ids, attrs.get("ref")?, "solid")?),
        "second"      => b.second   = Some(lookup(&self.solid_ids, attrs.get("ref")?, "solid")?),
        "position"    => b.position = attrs.vec3(length_unit(attrs.get_or("unit", "mm"))?)?,
        "rotation"    => b.angles   = attrs.vec3(angle_unit(attrs.get_or("unit", "rad"))?)?,
        "positionref" => b.position = *self.positions.get(attrs.get("ref")?)
                                        .ok_or_else(|| format!("Unknown position {}", attrs.get("ref").unwrap()))?,
        "rotationref" => b.angles   = *self.rotations.get(attrs.get("ref")?)
                                        .ok_or_else(|| format!("Unknown rotation {}", attrs.get("ref").unwrap()))?,
        _ => return Err(format!("Unsupported element <{tag}> in boolean solid {}", b.name)),
      },
      Some(_) => return Err(format!("Unexpected element <{tag}> in <solids>")),
    }
    Ok(())
  }

  fn structure_start(&mut self, tag : &str, attrs : &Attributes, open : &mut Option<Open>) -> Res<()> {
    match open.as_mut() {
      None => match tag {
        "volume" => {
          *open = Some(Open::Volume(VolumeBuilder {
            name      : attrs.get("name")?.to_string(),
            material  : None,
            solid     : None,
            daughters : Vec::new(),
            auxiliary : Vec::new(),
          }));
        }
        _ => return Err(format!("Unsupported element <{tag}> in <structure>")),
      },
      Some(Open::Volume(v)) => match tag {
        "materialref" => v.material = Some(lookup(&self.material_ids, attrs.get("ref")?, "material")?),
        "solidref"    => v.solid    = Some(lookup(&self.solid_ids, attrs.get("ref")?, "solid")?),
        "auxiliary"   => v.auxiliary.push((attrs.get("auxtype")?.to_string(),
                                           attrs.get_or("auxvalue", "").to_string())),
        "physvol"     => {
          let p = PhysvolBuilder {
            name       : attrs.get_or("name", "").to_string(),
            copynumber : attrs.get_or("copynumber", "0").parse::<i64>()
                           .map_err(|_| format!("Invalid copynumber {}", attrs.get_or("copynumber", "")))?,
            volume     : None,
            position   : DVec3::ZERO,
            angles     : DVec3::ZERO,
          };
          let Some(Open::Volume(v)) = open.take() else { unreachable!() };
          *open = Some(Open::Physvol(v, p));
        }
        _ => return Err(format!("Unsupported element <{tag}> in volume {}", v.name)),
      },
      Some(Open::Physvol(_, p)) => match tag {
        "volumeref"   => p.volume   = Some(lookup(&self.volume_ids, attrs.get("ref")?, "volume")?),
        "position"    => p.position = attrs.vec3(length_unit(attrs.get_or("unit", "mm"))?)?,
        "rotation"    => p.angles   = attrs.vec3(angle_unit(attrs.get_or("unit", "rad"))?)?,
        "positionref" => p.position = *self.positions.get(attrs.get("ref")?)
                                        .ok_or_else(|| format!("Unknown position {}", attrs.get("ref").unwrap()))?,
        "rotationref" => p.angles   = *self.rotations.get(attrs.get("ref")?)
                                        .ok_or_else(|| format!("Unknown rotation {}", attrs.get("ref").unwrap()))?,
        // scale (reflections) and file (external geometry)
        _ => return Err(format!("Unsupported element <{tag}> in physvol {}", p.name)),
      },
      Some(_) => return Err(format!("Unexpected element <{tag}> in <structure>")),
    }
    Ok(())
  }

  fn push_solid(&mut self, name : String, shape : Shape) {
    self.solid_ids.insert(name.clone(), self.solids.len());
    self.solids.push(Solid { name, shape });
  }
}

fn lookup(ids : &HashMap<String, usize>, name : &str, what : &str) -> Res<usize> {
  ids.get(name).copied().ok_or_else(|| format!("Reference to unknown {what} {name}"))
}

//------------------------------------------------------------

/// Attributes of an element as (key, value) strings.
/// Elements have only a few attributes, so a linear
/// search is fine.
struct Attributes {
  pairs : Vec<(String, String)>,
}

impl Attributes {
  fn new(e : &BytesStart) -> Res<Self> {
    let mut pairs = Vec::new();
    for attr in e.attributes() {
      let attr  = attr.map_err(|err| err.to_string())?;
      let key   = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
      let value = attr.unescape_value().map_err(|err| err.to_string())?.into_owned();
      pairs.push((key, value));
    }
    Ok(Self { pairs })
  }

  fn find(&self, key : &str) -> Option<&str> {
    self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
  }

  fn get(&self, key : &str) -> Res<&str> {
    self.find(key).ok_or_else(|| format!("Missing attribute {key}"))
  }

  fn get_or<'a>(&'a self, key : &str, default : &'a str) -> &'a str {
    self.find(key).unwrap_or(default)
  }

  fn f64(&self, key : &str) -> Res<f64> {
    let value = self.get(key)?;
    value.trim().parse::<f64>()
      .map_err(|_| format!("Attribute {key}=\"{value}\" is not a number (expressions are not supported)"))
  }

  fn f64_or(&self, key : &str, default : f64) -> Res<f64> {
    match self.find(key) {
      None    => Ok(default),
      Some(_) => self.f64(key),
    }
  }

  /// x, y, z (each defaulting to 0) multiplied by unit
  fn vec3(&self, unit : f64) -> Res<DVec3> {
    Ok(DVec3::new(self.f64_or("x", 0.0)?, self.f64_or("y", 0.0)?, self.f64_or("z", 0.0)?) * unit)
  }
}

/// in mm
fn length_unit(unit : &str) -> Res<f64> {
  match unit {
    "nm" => Ok(1e-6),
    "um" => Ok(1e-3),
    "mm" => Ok(1.0),
    "cm" => Ok(10.0),
    "m"  => Ok(1000.0),
    "km" => Ok(1e6),
    _    => Err(format!("Unknown length unit {unit}")),
  }
}

/// in rad
fn angle_unit(unit : &str) -> Res<f64> {
  match unit {
    "rad"            => Ok(1.0),
    "mrad"           => Ok(1e-3),
    "deg" | "degree" => Ok(std::f64::consts::PI/180.0),
    _                => Err(format!("Unknown angle unit {unit}")),
  }
}

/// in g/cm3
fn density_unit(unit : &str) -> Res<f64> {
  match unit {
    "g/cm3"  => Ok(1.0),
    "mg/cm3" => Ok(1e-3),
    "kg/m3"  => Ok(1e-3),
    "g/mm3"  => Ok(1e3),
    _        => Err(format!("Unknown density unit {unit}")),
  }
}

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  const GDML : &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<gdml>
  <define>
    <position name="v0" unit="mm" x="0" y="0" z="0"/>
    <position name="v1" unit="cm" x="1" y="0" z="0"/>
    <position name="v2" unit="mm" x="0" y="10" z="0"/>
    <position name="v3" unit="mm" x="0" y="0" z="10"/>
    <rotation name="r90" unit="deg" x="0" y="0" z="90"/>
  </define>
  <materials>
    <isotope N="1" Z="1" name="H1"><atom unit="g/mole" value="1.007"/></isotope>
    <material name="Vacuum" state="gas">
      <D unit="g/cm3" value="1e-25"/>
      <atom unit="g/mole" value="1.0"/>
    </material>
    <material name="Alu" Z="13">
      <T unit="K" value="293.15"/>
      <D unit="kg/m3" value="2699"/>
    </material>
  </materials>
  <solids>
    <tessellated aunit="deg" lunit="mm" name="Tet">
      <triangular vertex1="v0" vertex2="v2" vertex3="v1"/>
      <triangular vertex1="v0" vertex2="v1" vertex3="v3"/>
      <triangular vertex1="v0" vertex2="v3" vertex3="v2"/>
      <triangular vertex1="v1" vertex2="v2" vertex3="v3"/>
    </tessellated>
    <box lunit="cm" name="World" x="100" y="100" z="100"/>
    <tube aunit="deg" deltaphi="360" lunit="mm" name="Ring" rmax="50" rmin="48" startphi="0" z="2"/>
    <box lunit="mm" name="Slab" x="10" y="20" z="2"/>
    <intersection name="RingPart">
      <first ref="Ring"/>
      <second ref="Slab"/>
      <position name="RingPart_pos" unit="mm" x="0" y="36.5" z="0"/>
    </intersection>
  </solids>
  <structure>
    <volume name="TetVol">
      <materialref ref="Alu"/>
      <solidref ref="Tet"/>
      <auxiliary auxtype="SensDet" auxunit="mm" auxvalue="SimpleDet/ActiveDetectorSD"/>
    </volume>
    <volume name="RingVol">
      <materialref ref="Alu"/>
      <solidref ref="RingPart"/>
    </volume>
    <volume name="WorldVol">
      <materialref ref="Vacuum"/>
      <solidref ref="World"/>
      <physvol copynumber="7" name="tet0">
        <volumeref ref="TetVol"/>
        <position name="tet0_pos" unit="mm" x="100" y="0" z="0"/>
        <rotationref ref="r90"/>
      </physvol>
      <physvol name="ring0">
        <volumeref ref="RingVol"/>
        <position name="ring0_pos" unit="mm" x="0" y="0" z="-5"/>
      </physvol>
    </volume>
  </structure>
  <setup name="Default" version="1.0">
    <world ref="WorldVol"/>
  </setup>
</gdml>
"#;

  #[test]
  fn parse_example() {
    let geo = parse_gdml_str(GDML).unwrap();
    assert_eq!(geo.materials.len(), 2);
    assert!((geo.materials[1].density - 2.699).abs() < 1e-12);
    assert_eq!(geo.solids.len(), 5);
    let Shape::Tessellated { vertices, triangles } = &geo.solids[0].shape else { panic!() };
    // vertices are shared between the facets
    assert_eq!(vertices.len(), 4);
    assert_eq!(triangles.len(), 4);
    assert_eq!(vertices[2], DVec3::new(10.0, 0.0, 0.0));
    assert_eq!(geo.solids[1].shape, Shape::Box { half_lengths : DVec3::splat(500.0) });
    let Shape::Tube { half_z, delta_phi, .. } = geo.solids[2].shape else { panic!() };
    assert_eq!(half_z, 1.0);
    assert!((delta_phi - 2.0*std::f64::consts::PI).abs() < 1e-12);
    let Shape::Boolean { op, first, second, second_transform } = geo.solids[4].shape else { panic!() };
    assert_eq!((op, first, second), (BooleanOp::Intersection, 2, 3));
    assert_eq!(second_transform.translation, DVec3::new(0.0, 36.5, 0.0));

    let world = geo.world_volume();
    assert_eq!(world.name, "WorldVol");
    assert_eq!(world.daughters.len(), 2);
    assert_eq!(world.daughters[0].copynumber, 7);
    // ring0 has no copynumber, which defaults to 0
    assert_eq!(world.daughters[1].copynumber, 0);
    assert_eq!(geo.volumes[0].auxiliary[0].1, "SimpleDet/ActiveDetectorSD");

    let placements = geo.world_placements();
    assert_eq!(placements.len(), 3);
    // local x of the tetrahedron ends up along -y, see Transform::from_gdml
    let p = placements[1].transform.apply_point(DVec3::new(10.0, 0.0, 0.0));
    assert!((p - DVec3::new(100.0, -10.0, 0.0)).length() < 1e-9);
    assert_eq!(geo.density(placements[2].volume), 2.699);

    assert_eq!(geo.placements_by_copynumber.len(), 2);
    assert_eq!(geo.placement(7), Some(&placements[1]));
    assert_eq!(geo.placement(0).unwrap().volume, 1);
    assert_eq!(geo.placement(8), None);
  }

  #[test]
  fn duplicate_copynumber_is_an_error() {
    let gdml = GDML.replace(r#"<physvol name="ring0">"#, r#"<physvol copynumber="7" name="ring0">"#);
    assert!(matches!(parse_gdml_str(&gdml), Err(GeometryError::DuplicateCopynumber(7))));
  }

  #[test]
  fn cache_roundtrip() {
    let geo  = parse_gdml_str(GDML).unwrap();
    let path = std::env::temp_dir().join(format!("gaps-geometry-test-{}.bin", std::process::id()));
    geo.write_cache(&path).unwrap();
    let cached = Geometry::read_cache(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(geo, cached);
  }

  #[test]
  fn unsupported_is_an_error() {
    let gdml = GDML.replace(r#"<rotation name="r90""#, r#"<constant name="c" value="1"/><rotation name="r90""#);
    assert!(parse_gdml_str(&gdml).unwrap_err().to_string().contains("<constant>"));
    let gdml = GDML.replace(r#"x="100" y="100""#, r#"x="2*50" y="100""#);
    assert!(parse_gdml_str(&gdml).unwrap_err().to_string().contains("expressions"));
    let gdml = GDML.replace(r#"<volumeref ref="RingVol"/>"#, r#"<volumeref ref="Nope"/>"#);
    assert!(parse_gdml_str(&gdml).is_err());
  }
}
