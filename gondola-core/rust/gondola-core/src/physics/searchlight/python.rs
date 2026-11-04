//! Python bindings for searchlight
// This file is part of gaps-online-software and published
// under the GPLv3 license

use glam::DVec3;

use crate::prelude::*;

use super::*;

fn tuple(v : DVec3) -> (f64, f64, f64) {
  (v.x, v.y, v.z)
}

#[pymethods]
impl SearchlightGeometry {
  /// The detector geometry for searchlight
  ///
  /// # Arguments:
  ///   * paddles : dict paddle id -> TofPaddle, e.g. from
  ///               TofPaddle.all_as_dict()
  ///   * strips  : dict strip id -> TrackerStrip, e.g. from
  ///               TrackerStrip.all_as_dict(); needed for the
  ///               secondary tracks
  #[new]
  #[pyo3(signature = (paddles, strips = None))]
  fn new_py(paddles : HashMap<u8, TofPaddle>, strips : Option<HashMap<u32, TrackerStrip>>) -> Self {
    let geo = Self::new(&paddles);
    match strips {
      Some(st) => geo.with_strips(&st),
      None     => geo,
    }
  }

  fn __repr__(&self) -> String {
    format!("<SearchlightGeometry: {} paddles, {} overlapping pairs, {} strips>", self.paddles.len(), self.overlaps.len(), self.strips.len())
  }
}

#[pymethods]
impl SearchlightSettings {
  /// The settings of searchlight, with the defaults (all parts)
  #[new]
  fn new_py() -> Self {
    Self::default()
  }

  /// The settings without the independent vertex, the secondary
  /// tracks and the timing (phase A), with their weights
  #[staticmethod]
  #[pyo3(name="phase_a")]
  fn phase_a_py() -> Self {
    Self::phase_a()
  }

  /// For seeds which pass the flags: the vertex without the
  /// primary line, as an outer+vertex candidate if today's line has
  /// no primary-like tracker hits
  #[getter]
  #[pyo3(name="independent_vertex")]
  fn get_independent_vertex_py(&self) -> bool {
    self.independent_vertex
  }

  #[setter]
  #[pyo3(name="independent_vertex")]
  fn set_independent_vertex_py(&mut self, value : bool) {
    self.independent_vertex = value;
  }

  /// The RANSAC secondary track finder in the independent vertex
  #[getter]
  #[pyo3(name="use_tracks")]
  fn get_use_tracks_py(&self) -> bool {
    self.use_tracks
  }

  #[setter]
  #[pyo3(name="use_tracks")]
  fn set_use_tracks_py(&mut self, value : bool) {
    self.use_tracks = value;
  }

  /// The timing score in the independent vertex
  #[getter]
  #[pyo3(name="use_timing")]
  fn get_use_timing_py(&self) -> bool {
    self.use_timing
  }

  #[setter]
  #[pyo3(name="use_timing")]
  fn set_use_timing_py(&mut self, value : bool) {
    self.use_timing = value;
  }

  /// Hit cleaning of all candidate fits
  #[getter]
  #[pyo3(name="cleaning")]
  fn get_cleaning_py(&self) -> LineFitCleaning {
    self.cleaning
  }

  #[setter]
  #[pyo3(name="cleaning")]
  fn set_cleaning_py(&mut self, value : LineFitCleaning) {
    self.cleaning = value;
  }

  /// TOF hits below this edep (MeV) are not primary-like, a
  /// seed hit below it is flagged
  #[getter]
  #[pyo3(name="edep_min")]
  fn get_edep_min_py(&self) -> f64 {
    self.edep_min
  }

  #[setter]
  #[pyo3(name="edep_min")]
  fn set_edep_min_py(&mut self, value : f64) {
    self.edep_min = value;
  }

  /// Tracker hits above this energy (MeV) are primary-like and
  /// seed candidates
  #[getter]
  #[pyo3(name="trk_e_min")]
  fn get_trk_e_min_py(&self) -> f64 {
    self.trk_e_min
  }

  #[setter]
  #[pyo3(name="trk_e_min")]
  fn set_trk_e_min_py(&mut self, value : f64) {
    self.trk_e_min = value;
  }

  /// Error of the vertex as a fit point (mm)
  #[getter]
  #[pyo3(name="vertex_err")]
  fn get_vertex_err_py(&self) -> f64 {
    self.vertex_err
  }

  #[setter]
  #[pyo3(name="vertex_err")]
  fn set_vertex_err_py(&mut self, value : f64) {
    self.vertex_err = value;
  }

  #[getter]
  #[pyo3(name="use_vertex")]
  fn get_use_vertex_py(&self) -> bool {
    self.use_vertex
  }

  #[setter]
  #[pyo3(name="use_vertex")]
  fn set_use_vertex_py(&mut self, value : bool) {
    self.use_vertex = value;
  }

  #[getter]
  #[pyo3(name="use_tracker")]
  fn get_use_tracker_py(&self) -> bool {
    self.use_tracker
  }

  #[setter]
  #[pyo3(name="use_tracker")]
  fn set_use_tracker_py(&mut self, value : bool) {
    self.use_tracker = value;
  }

  #[getter]
  #[pyo3(name="use_trigger")]
  fn get_use_trigger_py(&self) -> bool {
    self.use_trigger
  }

  #[setter]
  #[pyo3(name="use_trigger")]
  fn set_use_trigger_py(&mut self, value : bool) {
    self.use_trigger = value;
  }

  /// Weights of the logistic regression over the candidate
  /// features. Empty: the most energy-weighted inliers win.
  #[getter]
  #[pyo3(name="weights")]
  fn get_weights_py(&self) -> Vec<f64> {
    self.weights.clone()
  }

  #[setter]
  #[pyo3(name="weights")]
  fn set_weights_py(&mut self, value : Vec<f64>) -> PyResult<()> {
    if !value.is_empty() && value.len() != SEARCHLIGHT_N_FEATURES {
      return Err(PyValueError::new_err(format!("Expected {} weights, got {}!", SEARCHLIGHT_N_FEATURES, value.len())));
    }
    self.weights = value;
    Ok(())
  }

  fn __repr__(&self) -> String {
    format!("<SearchlightSettings: edep_min {} MeV, trk_e_min {} MeV, vertex {}, tracker {}, trigger {}, independent vertex {} (tracks {}, timing {}), {} weights>",
            self.edep_min, self.trk_e_min, self.use_vertex, self.use_tracker, self.use_trigger,
            self.independent_vertex, self.use_tracks, self.use_timing, self.weights.len())
  }
}

#[pymethods]
impl SearchlightResult {
  /// True if a primary was reconstructed
  #[getter]
  #[pyo3(name="found")]
  fn get_found_py(&self) -> bool {
    self.best.is_some()
  }

  /// (outer, inner) seed hit: indices of the TOF hits
  #[getter]
  #[pyo3(name="seed")]
  fn get_seed_py(&self) -> Option<(usize, usize)> {
    self.seed
  }

  /// (outer seed hit flagged, inner seed hit flagged)
  #[getter]
  #[pyo3(name="flags")]
  fn get_flags_py(&self) -> (bool, bool) {
    self.flags
  }

  /// The vertex used for the candidates (mm)
  #[getter]
  #[pyo3(name="vertex")]
  fn get_vertex_py(&self) -> Option<(f64, f64, f64)> {
    self.vertex.map(tuple)
  }

  /// Quality of the independent vertex: (TOF hits agreeing on the
  /// stopping time, compatible measurements, from tracks, number
  /// of secondary tracks)
  #[getter]
  #[pyo3(name="vertex_quality")]
  fn get_vertex_quality_py(&self) -> Option<(f64, f64, f64, f64)> {
    self.vertex_quality.map(|q| (q[0], q[1], q[2], q[3]))
  }

  /// The kind of the chosen candidate
  #[getter]
  #[pyo3(name="candidate")]
  fn get_candidate_py(&self) -> Option<SearchlightCandidate> {
    self.primary().map(|c| c.kind)
  }

  /// Direction of the primary (unit vector, pointing inwards)
  #[getter]
  #[pyo3(name="direction")]
  fn get_direction_py(&self) -> Option<(f64, f64, f64)> {
    self.primary().map(|c| tuple(c.u))
  }

  /// A point on the primary line (mm)
  #[getter]
  #[pyo3(name="anchor")]
  fn get_anchor_py(&self) -> Option<(f64, f64, f64)> {
    self.primary().map(|c| tuple(c.a))
  }

  /// Probability of the chosen candidate from the selection
  #[getter]
  #[pyo3(name="probability")]
  fn get_probability_py(&self) -> Option<f64> {
    self.primary().map(|c| c.prob)
  }

  /// Hits accepted in the fit of the chosen candidate (TOF
  /// hits first, then tracker hits; an extra point such as the
  /// vertex has the index of the number of hits)
  #[getter]
  #[pyo3(name="accepted")]
  fn get_accepted_py(&self) -> Vec<usize> {
    self.primary().map(|c| c.hits.clone()).unwrap_or_default()
  }

  /// All candidates as (kind, direction, probability, features,
  /// allowed), e.g. to retrain the weights (see
  /// gondola.reconstruction.searchlight_training). Allowed: the
  /// candidate took part in the selection.
  #[getter]
  #[pyo3(name="candidates")]
  fn get_candidates_py(&self) -> Vec<(SearchlightCandidate, (f64, f64, f64), f64, Vec<f64>, bool)> {
    self.candidates.iter().map(|c| (c.kind, tuple(c.u), c.prob, c.features.clone(), c.allowed)).collect()
  }

  fn __repr__(&self) -> String {
    format!("{}", self)
  }
}

#[pymethods]
impl Searchlight {
  /// The searchlight reconstruction of the primary track
  ///
  /// # Arguments:
  ///   * geometry : SearchlightGeometry
  ///   * settings : SearchlightSettings (default if None)
  #[new]
  #[pyo3(signature = (geometry, settings = None))]
  fn new_py(geometry : SearchlightGeometry, settings : Option<SearchlightSettings>) -> Self {
    Self::new(geometry, settings.unwrap_or_default())
  }

  #[getter]
  #[pyo3(name="settings")]
  fn get_settings_py(&self) -> SearchlightSettings {
    self.settings.clone()
  }

  #[setter]
  #[pyo3(name="settings")]
  fn set_settings_py(&mut self, value : SearchlightSettings) {
    self.settings = value;
  }

  /// Reconstruct the primary of a TelemetryEvent. The TOF hit
  /// times have to be normalized (tof_normalize_hit_times), the
  /// tracker hits calibrated (energy in MeV).
  ///
  /// # Arguments:
  ///   * event     : TelemetryEvent
  ///   * triggered : paddle ids of triggered paddles without a hit
  #[pyo3(name="reconstruct", signature = (event, triggered = None))]
  fn reconstruct_py(&self, event : &TelemetryEvent, triggered : Option<Vec<u8>>) -> SearchlightResult {
    self.reconstruct(event, &triggered.unwrap_or_default())
  }

  /// Reconstruct the primary from hits given as lists (mm, ns,
  /// MeV), e.g. for studies on cached events
  ///
  /// # Arguments:
  ///   * tof_pos        : TOF hit positions (x, y, z)
  ///   * tof_t          : normalized TOF hit times
  ///   * tof_edep       : TOF edep
  ///   * tof_paddle     : TOF paddle ids
  ///   * trk_pos        : tracker hit positions (x, y, z)
  ///   * trk_e          : tracker hit energies
  ///   * trk_strip      : strip ids of the tracker hits (for the
  ///                      secondary tracks)
  ///   * triggered      : paddle ids of triggered paddles without a hit
  ///   * merge_overlaps : merge TOF hits of overlapping paddles
  #[pyo3(name="reconstruct_hits", signature = (tof_pos, tof_t, tof_edep, tof_paddle, trk_pos, trk_e, trk_strip = None, triggered = None, merge_overlaps = true))]
  fn reconstruct_hits_py(&self,
                         tof_pos        : Vec<(f64, f64, f64)>,
                         tof_t          : Vec<f64>,
                         tof_edep       : Vec<f64>,
                         tof_paddle     : Vec<u8>,
                         trk_pos        : Vec<(f64, f64, f64)>,
                         trk_e          : Vec<f64>,
                         trk_strip      : Option<Vec<u32>>,
                         triggered      : Option<Vec<u8>>,
                         merge_overlaps : bool) -> PyResult<SearchlightResult> {
    let ntof = tof_pos.len();
    let trk_strip = trk_strip.unwrap_or_else(|| vec![0; trk_pos.len()]);
    if tof_t.len() != ntof || tof_edep.len() != ntof || tof_paddle.len() != ntof || trk_e.len() != trk_pos.len() || trk_strip.len() != trk_pos.len() {
      return Err(PyValueError::new_err("The hit lists have different lengths!"));
    }
    let tof : Vec<(DVec3, f64, f64, u8)> = (0..ntof).map(|k| {
      let p = tof_pos[k];
      (DVec3::new(p.0, p.1, p.2), tof_t[k], tof_edep[k], tof_paddle[k])
    }).collect();
    let trk : Vec<(DVec3, f64, u32)> = (0..trk_pos.len()).map(|k| {
      let p = trk_pos[k];
      (DVec3::new(p.0, p.1, p.2), trk_e[k], trk_strip[k])
    }).collect();
    let mut hits = SearchlightHits::new(&tof, &trk, Some(&self.geometry));
    if !merge_overlaps {
      // the positions as given
      for (k, h) in tof.iter().enumerate() {
        hits.pos[k] = h.0;
      }
    }
    Ok(self.reconstruct_hits(&hits, &triggered.unwrap_or_default()))
  }

  fn __repr__(&self) -> String {
    format!("<Searchlight: {}>", self.settings.__repr__())
  }
}
