//! The data model of the parsed geometry. All lengths are
//! in mm, all angles in rad and densities in g/cm3.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use crate::prelude::*;

use std::io::BufWriter;

use glam::{
  DMat3,
  DVec3,
};
use serde::{
  Deserialize,
  Serialize,
};

#[cfg(feature="pybindings")]
use numpy::{
  IntoPyArray,
  ndarray::Array2,
};

use crate::geometry::gdml::parse_gdml_file;
use crate::geometry::column_density::LazyRaycaster;
#[cfg(feature="pybindings")]
use crate::geometry::column_density::TrackPath;
#[cfg(feature="pybindings")]
use crate::physics::Tracklet;

/// Identifies the cache file and its layout. Bump the
/// version whenever the data model changes.
const CACHE_MAGIC   : &[u8; 8] = b"GAPSGEO\0";
const CACHE_VERSION : u32      = 1;

/// Index of a material in Geometry.materials
pub type MaterialId = usize;
/// Index of a solid in Geometry.solids
pub type SolidId    = usize;
/// Index of a logical volume in Geometry.volumes, in the
/// order the volumes are defined in the GDML file. It is
/// assigned by the parser and is not a Geant4 number.
pub type VolumeId   = usize;

//------------------------------------------------------------

/// Rigid transformation from a local (daughter) frame into
/// the frame of its parent
///
/// p_parent = rotation * p_local + translation
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Transform {
  pub rotation    : DMat3,
  pub translation : DVec3,
}

impl Transform {
  pub const IDENTITY : Self = Self {
    rotation    : DMat3::IDENTITY,
    translation : DVec3::ZERO,
  };

  /// Transform as built by Geant4 from a GDML position and
  /// rotation (G4GDMLReadStructure::PhysvolRead and
  /// G4GDMLReadSolids::BooleanRead).
  ///
  /// Geant4 builds the rotation matrix R by rotating about
  /// x, then y, then z (R = Rz * Ry * Rx) and places the
  /// daughter with the inverse of that matrix.
  ///
  /// # Arguments:
  ///   * position : in mm
  ///   * angles   : rotation angles about x, y, z in rad
  pub fn from_gdml(position : DVec3, angles : DVec3) -> Self {
    let r = DMat3::from_rotation_z(angles.z)
          * DMat3::from_rotation_y(angles.y)
          * DMat3::from_rotation_x(angles.x);
    Self {
      rotation    : r.transpose(),
      translation : position,
    }
  }

  /// local -> parent
  pub fn apply_point(&self, p : DVec3) -> DVec3 {
    self.rotation * p + self.translation
  }

  /// local -> parent (directions are only rotated)
  pub fn apply_vector(&self, v : DVec3) -> DVec3 {
    self.rotation * v
  }

  /// parent -> local
  pub fn inverse_apply_point(&self, p : DVec3) -> DVec3 {
    self.rotation.transpose() * (p - self.translation)
  }

  /// parent -> local
  pub fn inverse_apply_vector(&self, v : DVec3) -> DVec3 {
    self.rotation.transpose() * v
  }
}

impl fmt::Display for Transform {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Transform");
    for k in 0..3 {
      let row = self.rotation.row(k);
      repr += &(format!("\n  | {:8.4} {:8.4} {:8.4} |", row.x, row.y, row.z));
    }
    repr += &(format!("\n  translation {:.2} {:.2} {:.2}>",
                      self.translation.x, self.translation.y, self.translation.z));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Transform {
  /// The rotation matrix, as a list of rows
  #[getter]
  fn get_rotation(&self) -> [[f64; 3]; 3] {
    [self.rotation.row(0).to_array(),
     self.rotation.row(1).to_array(),
     self.rotation.row(2).to_array()]
  }

  /// Translation in mm
  #[getter]
  fn get_translation(&self) -> (f64, f64, f64) {
    self.translation.into()
  }

  /// Transform a point from the local into the parent frame
  #[pyo3(name="apply_point")]
  fn apply_point_py(&self, p : (f64, f64, f64)) -> (f64, f64, f64) {
    self.apply_point(p.into()).into()
  }

  /// Transform a direction from the local into the parent frame
  #[pyo3(name="apply_vector")]
  fn apply_vector_py(&self, v : (f64, f64, f64)) -> (f64, f64, f64) {
    self.apply_vector(v.into()).into()
  }

  /// Transform a point from the parent into the local frame
  #[pyo3(name="inverse_apply_point")]
  fn inverse_apply_point_py(&self, p : (f64, f64, f64)) -> (f64, f64, f64) {
    self.inverse_apply_point(p.into()).into()
  }

  /// Transform a direction from the parent into the local frame
  #[pyo3(name="inverse_apply_vector")]
  fn inverse_apply_vector_py(&self, v : (f64, f64, f64)) -> (f64, f64, f64) {
    self.inverse_apply_vector(v.into()).into()
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Transform);

//------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Material {
  pub name    : String,
  /// g/cm3
  pub density : f64,
}

impl fmt::Display for Material {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Material {} : {:.6e} g/cm3>", self.name, self.density)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Material {
  #[getter]
  fn get_name(&self) -> String {
    self.name.clone()
  }

  /// Density in g/cm3
  #[getter]
  fn get_density(&self) -> f64 {
    self.density
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Material);

//------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
  Union,
  Intersection,
  Subtraction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shape {
  /// Closed triangle mesh, vertices in the local frame
  /// of the solid. Triangles are indices into vertices.
  Tessellated {
    vertices  : Vec<DVec3>,
    triangles : Vec<[u32; 3]>,
  },
  /// Box centered at the origin
  Box {
    half_lengths : DVec3,
  },
  /// (Segment of a) tube along z, centered at the origin
  Tube {
    rmin        : f64,
    rmax        : f64,
    half_z      : f64,
    start_phi   : f64,
    delta_phi   : f64,
  },
  /// The second solid is placed in the frame of the first
  /// one with second_transform
  Boolean {
    op               : BooleanOp,
    first            : SolidId,
    second           : SolidId,
    second_transform : Transform,
  },
}

/// The type of a solid, mainly to inspect
/// the geometry from python
#[derive(Debug, Copy, Clone, PartialEq, FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum SolidKind {
  Unknown      = 0u8,
  Tessellated  = 10u8,
  Box          = 20u8,
  Tube         = 30u8,
  Union        = 40u8,
  Intersection = 41u8,
  Subtraction  = 42u8,
}

// expand it so that it can be used as keys 
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl SolidKind {
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  }
}

expand_and_test_enum!(SolidKind, test_solidkind_repr);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Solid {
  pub name  : String,
  pub shape : Shape,
}

impl Solid {
  pub fn kind(&self) -> SolidKind {
    match self.shape {
      Shape::Tessellated {..} => SolidKind::Tessellated,
      Shape::Box {..}         => SolidKind::Box,
      Shape::Tube {..}        => SolidKind::Tube,
      Shape::Boolean {op, ..} => match op {
        BooleanOp::Union        => SolidKind::Union,
        BooleanOp::Intersection => SolidKind::Intersection,
        BooleanOp::Subtraction  => SolidKind::Subtraction,
      }
    }
  }

  /// Number of vertices of a tessellated solid, 0 otherwise
  pub fn n_vertices(&self) -> usize {
    match &self.shape {
      Shape::Tessellated { vertices, .. } => vertices.len(),
      _ => 0
    }
  }

  /// Number of triangles of a tessellated solid, 0 otherwise
  pub fn n_triangles(&self) -> usize {
    match &self.shape {
      Shape::Tessellated { triangles, .. } => triangles.len(),
      _ => 0
    }
  }
}

impl fmt::Display for Solid {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<Solid {} ({})", self.name, self.kind().as_ref());
    match &self.shape {
      Shape::Tessellated { vertices, triangles } => {
        repr += &(format!("\n  {} vertices, {} triangles", vertices.len(), triangles.len()));
      }
      Shape::Box { half_lengths } => {
        repr += &(format!("\n  half lengths {:.2} {:.2} {:.2}", half_lengths.x, half_lengths.y, half_lengths.z));
      }
      Shape::Tube { rmin, rmax, half_z, start_phi, delta_phi } => {
        repr += &(format!("\n  rmin {:.2} rmax {:.2} half z {:.2} phi {:.4} + {:.4}",
                          rmin, rmax, half_z, start_phi, delta_phi));
      }
      Shape::Boolean { first, second, .. } => {
        repr += &(format!("\n  first {} second {}", first, second));
      }
    }
    write!(f, "{}>", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Solid {
  #[getter]
  fn get_name(&self) -> String {
    self.name.clone()
  }

  #[getter]
  #[pyo3(name="kind")]
  fn get_kind_py(&self) -> SolidKind {
    self.kind()
  }

  #[getter]
  #[pyo3(name="n_vertices")]
  fn get_n_vertices_py(&self) -> usize {
    self.n_vertices()
  }

  #[getter]
  #[pyo3(name="n_triangles")]
  fn get_n_triangles_py(&self) -> usize {
    self.n_triangles()
  }

  /// Vertices of a tessellated solid in its local frame
  /// as an (N,3) array in mm. Empty for other solids.
  #[getter]
  fn get_vertices<'py>(&self, py : Python<'py>) -> Bound<'py, PyArray2<f64>> {
    let flat : Vec<f64> = match &self.shape {
      Shape::Tessellated { vertices, .. } => vertices.iter().flat_map(|v| v.to_array()).collect(),
      _ => Vec::new()
    };
    let n = flat.len()/3;
    Array2::from_shape_vec((n, 3), flat).unwrap().into_pyarray(py)
  }

  /// Triangles of a tessellated solid as an (N,3) array
  /// of indices into vertices. Empty for other solids.
  #[getter]
  fn get_triangles<'py>(&self, py : Python<'py>) -> Bound<'py, PyArray2<u32>> {
    let flat : Vec<u32> = match &self.shape {
      Shape::Tessellated { triangles, .. } => triangles.iter().flatten().copied().collect(),
      _ => Vec::new()
    };
    let n = flat.len()/3;
    Array2::from_shape_vec((n, 3), flat).unwrap().into_pyarray(py)
  }

  /// Half lengths in mm of a box, None for other solids
  #[getter]
  fn get_half_lengths(&self) -> Option<(f64, f64, f64)> {
    match self.shape {
      Shape::Box { half_lengths } => Some(half_lengths.into()),
      _ => None
    }
  }

  /// (rmin, rmax, half_z, start_phi, delta_phi) of a tube
  /// in mm and rad, None for other solids
  #[getter]
  fn get_tube(&self) -> Option<(f64, f64, f64, f64, f64)> {
    match self.shape {
      Shape::Tube { rmin, rmax, half_z, start_phi, delta_phi } => Some((rmin, rmax, half_z, start_phi, delta_phi)),
      _ => None
    }
  }

  /// Indices of the first and second solid of a boolean
  /// solid, None for other solids
  #[getter]
  fn get_operands(&self) -> Option<(SolidId, SolidId)> {
    match self.shape {
      Shape::Boolean { first, second, .. } => Some((first, second)),
      _ => None
    }
  }

  /// Placement of the second solid in the frame of the first
  /// one for a boolean solid, None for other solids
  #[getter]
  fn get_second_transform(&self) -> Option<Transform> {
    match self.shape {
      Shape::Boolean { second_transform, .. } => Some(second_transform),
      _ => None
    }
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Solid);

//------------------------------------------------------------

/// A placement of a (logical) volume inside its mother volume
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Placement {
  pub name       : String,
  /// The Geant4 copy number of the placement (copynumber
  /// attribute of the physvol, 0 if not given)
  pub copynumber : i64,
  /// the placed logical volume (index in Geometry.volumes)
  pub volume     : VolumeId,
  /// daughter -> mother frame
  pub transform  : Transform,
}

impl fmt::Display for Placement {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Placement {} (copynumber {}, volume {})>", self.name, self.copynumber, self.volume)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Placement {
  #[getter]
  fn get_name(&self) -> String {
    self.name.clone()
  }

  /// The Geant4 copy number of the placement (copynumber
  /// attribute of the physvol, 0 if not given)
  #[getter]
  fn get_copynumber(&self) -> i64 {
    self.copynumber
  }

  /// Index of the placed logical volume in Geometry.volumes,
  /// i.e. geo.volumes[p.volume] is the volume which is placed.
  /// The index follows the order of the volumes in the GDML
  /// file and is not a Geant4 number. A logical volume can be
  /// placed many times, so this does not identify the placement
  /// (use the copynumber for that).
  #[getter]
  fn get_volume(&self) -> VolumeId {
    self.volume
  }

  /// daughter -> mother frame
  #[getter]
  fn get_transform(&self) -> Transform {
    self.transform
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Placement);

//------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Volume {
  pub name      : String,
  pub material  : MaterialId,
  pub solid     : SolidId,
  pub daughters : Vec<Placement>,
  /// GDML auxiliary information as (auxtype, auxvalue),
  /// e.g. ("SensDet", "SimpleDet/ActiveDetectorSD")
  pub auxiliary : Vec<(String, String)>,
}

impl fmt::Display for Volume {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Volume {} (material {}, solid {}, {} daughters)>",
           self.name, self.material, self.solid, self.daughters.len())
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Volume {
  #[getter]
  fn get_name(&self) -> String {
    self.name.clone()
  }

  /// Index of the material in Geometry.materials
  #[getter]
  fn get_material(&self) -> MaterialId {
    self.material
  }

  /// Index of the solid in Geometry.solids
  #[getter]
  fn get_solid(&self) -> SolidId {
    self.solid
  }

  #[getter]
  fn get_daughters(&self) -> Vec<Placement> {
    self.daughters.clone()
  }

  /// GDML auxiliary information as (auxtype, auxvalue)
  #[getter]
  fn get_auxiliary(&self) -> Vec<(String, String)> {
    self.auxiliary.clone()
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Volume);

//------------------------------------------------------------

/// A placed volume with its transformation into the
/// world frame
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct WorldPlacement {
  /// the placed logical volume (index in Geometry.volumes)
  pub volume     : VolumeId,
  /// the Geant4 copy number of the placement, None for
  /// the world
  pub copynumber : Option<i64>,
  /// the logical volume this one is placed in (index in
  /// Geometry.volumes), None for the world
  pub mother     : Option<VolumeId>,
  /// local -> world frame
  pub transform : Transform,
  /// nesting depth, 0 for the world volume
  pub depth     : usize,
}

impl fmt::Display for WorldPlacement {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<WorldPlacement volume {} (copynumber {:?}, mother {:?}, depth {})>",
           self.volume, self.copynumber, self.mother, self.depth)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl WorldPlacement {
  /// Index of the placed logical volume in Geometry.volumes,
  /// i.e. geo.volumes[p.volume] is the volume which is placed.
  /// The index follows the order of the volumes in the GDML
  /// file and is not a Geant4 number. A logical volume can be
  /// placed many times, so this does not identify the placement
  /// (use the copynumber for that).
  #[getter]
  fn get_volume(&self) -> VolumeId {
    self.volume
  }

  /// The Geant4 copy number of the placement, None for
  /// the world
  #[getter]
  fn get_copynumber(&self) -> Option<i64> {
    self.copynumber
  }

  /// Index of the logical volume this one is placed in
  /// (in Geometry.volumes), None for the world
  #[getter]
  fn get_mother(&self) -> Option<VolumeId> {
    self.mother
  }

  /// local -> world frame
  #[getter]
  fn get_transform(&self) -> Transform {
    self.transform
  }

  /// Nesting depth, 0 for the world volume
  #[getter]
  fn get_depth(&self) -> usize {
    self.depth
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(WorldPlacement);

//------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Geometry {
  pub materials : Vec<Material>,
  pub solids    : Vec<Solid>,
  pub volumes   : Vec<Volume>,
  /// the world volume (index in volumes)
  pub world     : VolumeId,
  /// All placements (except the world) by their Geant4
  /// copy number. It is not stored in the cache, but
  /// rebuilt when the geometry is created.
  #[serde(skip)]
  pub placements_by_copynumber : HashMap<i64, WorldPlacement>,
  /// for the column density, built when first needed
  #[serde(skip)]
  pub(crate) raycaster : LazyRaycaster,
}

impl Geometry {
  /// Create the geometry and index its placements by their
  /// Geant4 copy number, which have to be unique
  pub fn new(materials : Vec<Material>,
             solids    : Vec<Solid>,
             volumes   : Vec<Volume>,
             world     : VolumeId) -> Result<Self, GeometryError> {
    let mut geo = Self {
      materials,
      solids,
      volumes,
      world,
      placements_by_copynumber : HashMap::new(),
      raycaster                : LazyRaycaster::default(),
    };
    geo.index_copynumbers()?;
    Ok(geo)
  }

  fn index_copynumbers(&mut self) -> Result<(), GeometryError> {
    let mut index = HashMap::new();
    for p in self.world_placements() {
      let Some(copynumber) = p.copynumber else { continue };
      if index.insert(copynumber, p).is_some() {
        return Err(GeometryError::DuplicateCopynumber(copynumber));
      }
    }
    self.placements_by_copynumber = index;
    Ok(())
  }

  /// Load the geometry, either from a GDML file or from a
  /// binary cache. If cache is given, it will be used if it
  /// is newer than the GDML file, otherwise the GDML file is
  /// parsed and the cache (re)written.
  pub fn load(gdml : &Path, cache : Option<&Path>) -> Result<Self, GeometryError> {
    let Some(cache) = cache else {
      return parse_gdml_file(gdml);
    };
    let modified = |p : &Path| fs::metadata(p).and_then(|m| m.modified()).ok();
    if let (Some(t_cache), Some(t_gdml)) = (modified(cache), modified(gdml)) {
      if t_cache >= t_gdml {
        return Self::read_cache(cache);
      }
    }
    let geo = parse_gdml_file(gdml)?;
    geo.write_cache(cache)?;
    Ok(geo)
  }

  pub fn write_cache(&self, path : &Path) -> Result<(), GeometryError> {
    let mut f = BufWriter::new(File::create(path)?);
    f.write_all(CACHE_MAGIC)?;
    f.write_all(&CACHE_VERSION.to_le_bytes())?;
    postcard::to_io(self, &mut f)?;
    f.flush()?;
    Ok(())
  }

  pub fn read_cache(path : &Path) -> Result<Self, GeometryError> {
    let mut bytes = Vec::new();
    BufReader::new(File::open(path)?).read_to_end(&mut bytes)?;
    let header = CACHE_MAGIC.len() + 4;
    if bytes.len() < header || &bytes[..CACHE_MAGIC.len()] != CACHE_MAGIC {
      return Err(GeometryError::Cache(format!("{} is not a geometry cache file", path.display())));
    }
    let version = u32::from_le_bytes(bytes[CACHE_MAGIC.len()..header].try_into().unwrap());
    if version != CACHE_VERSION {
      return Err(GeometryError::Cache(format!("Cache version {} is not supported (expected {})", version, CACHE_VERSION)));
    }
    let mut geo : Self = postcard::from_bytes(&bytes[header..])?;
    geo.index_copynumbers()?;
    Ok(geo)
  }

  pub fn world_volume(&self) -> &Volume {
    &self.volumes[self.world]
  }

  /// Density of the material of a volume in g/cm3
  pub fn density(&self, volume : VolumeId) -> f64 {
    self.materials[self.volumes[volume].material].density
  }

  /// The placement with the given Geant4 copy number
  pub fn placement(&self, copynumber : i64) -> Option<&WorldPlacement> {
    self.placements_by_copynumber.get(&copynumber)
  }

  /// All placed volumes (including the world volume itself)
  /// with their transformation into the world frame
  pub fn world_placements(&self) -> Vec<WorldPlacement> {
    let mut placements = vec![WorldPlacement {
      volume     : self.world,
      copynumber : None,
      mother     : None,
      transform  : Transform::IDENTITY,
      depth      : 0,
    }];
    // the list grows while we walk it, which makes
    // this a breadth first traversal
    let mut k = 0;
    while k < placements.len() {
      let mother = placements[k].clone();
      for d in &self.volumes[mother.volume].daughters {
        placements.push(WorldPlacement {
          volume     : d.volume,
          copynumber : Some(d.copynumber),
          mother     : Some(mother.volume),
          transform  : Transform {
            rotation    : mother.transform.rotation * d.transform.rotation,
            translation : mother.transform.apply_point(d.transform.translation),
          },
          depth      : mother.depth + 1,
        });
      }
      k += 1;
    }
    placements
  }
}

impl fmt::Display for Geometry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Geometry");
    repr += &(format!("\n  {} materials, {} solids, {} volumes",
                      self.materials.len(), self.solids.len(), self.volumes.len()));
    repr += &(format!("\n  world {} ({} placements)>",
                      self.world_volume().name, self.world_volume().daughters.len()));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Geometry {
  /// Load the geometry, either from a GDML file or from a
  /// binary cache. If cache is given, it will be used if it
  /// is newer than the GDML file, otherwise the GDML file is
  /// parsed and the cache (re)written.
  ///
  /// # Arguments:
  ///   * gdml  : path to the GDML file
  ///   * cache : path to the binary cache file
  #[staticmethod]
  #[pyo3(name="load", signature = (gdml, cache=None))]
  fn load_py(gdml : String, cache : Option<String>) -> PyResult<Self> {
    Ok(Self::load(Path::new(&gdml), cache.as_deref().map(Path::new))?)
  }

  /// Parse a GDML file
  #[staticmethod]
  fn from_gdml(path : String) -> PyResult<Self> {
    Ok(parse_gdml_file(Path::new(&path))?)
  }

  /// Read the geometry from a binary cache file
  #[staticmethod]
  fn from_cache(path : String) -> PyResult<Self> {
    Ok(Self::read_cache(Path::new(&path))?)
  }

  /// Write the geometry to a binary cache file
  #[pyo3(name="write_cache")]
  fn write_cache_py(&self, path : String) -> PyResult<()> {
    Ok(self.write_cache(Path::new(&path))?)
  }

  #[getter]
  fn get_materials(&self) -> Vec<Material> {
    self.materials.clone()
  }

  #[getter]
  fn get_solids(&self) -> Vec<Solid> {
    self.solids.clone()
  }

  #[getter]
  fn get_volumes(&self) -> Vec<Volume> {
    self.volumes.clone()
  }

  /// Index of the world volume in volumes
  #[getter]
  fn get_world(&self) -> VolumeId {
    self.world
  }

  #[getter]
  #[pyo3(name="world_volume")]
  fn get_world_volume_py(&self) -> Volume {
    self.world_volume().clone()
  }

  /// Density of the material of a volume in g/cm3
  #[pyo3(name="density")]
  fn density_py(&self, volume : VolumeId) -> PyResult<f64> {
    if volume >= self.volumes.len() {
      return Err(PyValueError::new_err(format!("No volume with index {volume}")));
    }
    Ok(self.density(volume))
  }

  /// All placements (except the world) by their Geant4
  /// copy number. This copies all placements, for a single
  /// lookup use placement(copynumber)
  #[getter]
  #[pyo3(name="placements_by_copynumber")]
  fn get_placements_by_copynumber_py(&self) -> HashMap<i64, WorldPlacement> {
    self.placements_by_copynumber.clone()
  }

  /// The placement with the given Geant4 copy number,
  /// None if there is no such placement
  #[pyo3(name="placement")]
  fn placement_py(&self, copynumber : i64) -> Option<WorldPlacement> {
    self.placement(copynumber).cloned()
  }

  /// All placed volumes (including the world volume itself)
  /// with their transformation into the world frame
  #[pyo3(name="world_placements")]
  fn world_placements_py(&self) -> Vec<WorldPlacement> {
    self.world_placements()
  }

  /// The volumes along the straight track a -> b
  ///
  /// # Arguments:
  ///   * a, b : start and stop point (x, y, z) in mm
  #[pyo3(name="trace")]
  fn trace_py(&self, a : (f64, f64, f64), b : (f64, f64, f64)) -> TrackPath {
    self.trace(a.into(), b.into())
  }

  /// Column density along the straight track a -> b in
  /// g/cm2, including the air of the world volume
  ///
  /// # Arguments:
  ///   * a, b : start and stop point (x, y, z) in mm
  #[pyo3(name="column_density")]
  fn column_density_py(&self, a : (f64, f64, f64), b : (f64, f64, f64)) -> f64 {
    self.column_density(a.into(), b.into())
  }

  /// Cumulative column density in g/cm2 at each point of a
  /// track made of straight segments between the points.
  /// The first value is always 0.
  ///
  /// # Arguments:
  ///   * points : list of (x, y, z) in mm
  #[pyo3(name="depths")]
  fn depths_py(&self, points : Vec<(f64, f64, f64)>) -> Vec<f64> {
    let points : Vec<DVec3> = points.into_iter().map(DVec3::from).collect();
    self.depths(&points)
  }

  /// The start and stop point of a tracklet in mm. For an
  /// infinite tracklet, the track goes from the start along
  /// the vertex momentum until it leaves the world volume.
  #[pyo3(name="tracklet_endpoints")]
  fn tracklet_endpoints_py(&self, tracklet : &Tracklet) -> PyResult<((f64, f64, f64), (f64, f64, f64))> {
    let (start, stop) = self.tracklet_endpoints(tracklet)?;
    Ok((start.into(), stop.into()))
  }

  /// The volumes along a tracklet, see tracklet_endpoints
  #[pyo3(name="trace_tracklet")]
  fn trace_tracklet_py(&self, tracklet : &Tracklet) -> PyResult<TrackPath> {
    Ok(self.trace_tracklet(tracklet)?)
  }

  /// Column density along a tracklet in g/cm2, including
  /// the air of the world volume, see tracklet_endpoints
  #[pyo3(name="tracklet_column_density")]
  fn tracklet_column_density_py(&self, tracklet : &Tracklet) -> PyResult<f64> {
    Ok(self.tracklet_column_density(tracklet)?)
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Geometry);

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use std::f64::consts::FRAC_PI_2;

  #[test]
  fn transform_roundtrip() {
    let t = Transform::from_gdml(DVec3::new(1.0, 2.0, 3.0), DVec3::new(0.3, -1.1, 2.0));
    let p = DVec3::new(-4.0, 5.0, 0.5);
    assert!((t.inverse_apply_point(t.apply_point(p)) - p).length() < 1e-12);
  }

  #[test]
  fn transform_g4_convention() {
    // a GDML rotation of +90 deg about z places the
    // daughter rotated by -90 deg, so the local x axis
    // ends up along -y in the mother
    let t = Transform::from_gdml(DVec3::ZERO, DVec3::new(0.0, 0.0, FRAC_PI_2));
    assert!((t.apply_vector(DVec3::X) - DVec3::NEG_Y).length() < 1e-12);
  }
}
