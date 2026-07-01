// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McTree {
  //pub tracks    : Vec<McTrack>,
  pub trackmap  : HashMap<u32, McTrack>,
  pub max_gen   : usize,

  //pub track_map : HashMap<usize, McTrack>,
  ////pub n_tracks : usize;
  //  // number of generations in the tree
  //  uint NGenerations() const;
  //  // number of tracks in total
  //  uint NTracks() const;
  //  // the initial track of the priamry
  //  CTrackMc GetPrimary() const;
  //  // daughters of any track in the tree
  //  std::vector<CTrackMc> GetDaughters(CTrackMc track) const;
  //  // get a specific generation
  //  std::vector<CTrackMc> GetGeneration(uint gen) const;
  //  // access to the whole tree
  //  const std::map<uint, std::vector<CTrackMc>>& GetTree() const;


  //private:
  //  void InitializeTree(std::vector<CTrackMc> tracks);
  //  void MakeGenerationTree();
  //  bool StillAlive(CTrackMc track) const;
  //  uint maxGen_;
  //  uint nTracks_;
  //  std::map<uint, CTrackMc> tracks_;
  //  std::map<uint, std::vector<CTrackMc>> generationTree_;
  //  CTrackMc primary_;
  //};
}

impl McTree {
  pub fn new() -> Self {
    Self {
      //tracks   : Vec::<McTrack>::new(),
      max_gen  : 0,
      trackmap : HashMap::<u32, McTrack>::new(),
    }
  }

  pub fn get_daughters(&self) -> Vec<McTrack> {
    let mut daughters =  Vec::<McTrack>::new();
  //for (auto t_pair : tracks_)
  //  {
  //    if (t_pair.second.GetParentId() == track.GetTrackId())
  //      {daughters.push_back(t_pair.second);}
  //  }
    return daughters;
  }

  // create all the tracks and create the actual 
  // tree
  pub fn assemble(&mut self, hits : &mut Vec<McHit>) {
    let mut trackmap = HashMap::<u32, McTrack>::new();
    let mut nhits = hits.len();
    while nhits > 0 {
      let h = hits.pop().unwrap();
      if trackmap.contains_key(&h.track_id) {
        trackmap.get_mut(&h.track_id).unwrap().hits.push(h);
      } else {
        let mut new_track   = McTrack::new();
        new_track.track.pdg = h.pdg;
        new_track.hits      = vec![h];
        trackmap.insert(h.track_id, new_track);
      }
      nhits -= 1;
    }
    let mut tm_keys = Vec::<u32>::new();
    for k in trackmap.keys() {
      tm_keys.push(*k);
    }
    for k in &tm_keys {
      trackmap.get_mut(&k).unwrap().hits.sort_by(|i,j| i.glob_time.total_cmp(&j.glob_time));  
    }
    self.trackmap = trackmap;
  }
}

impl fmt::Display for McTree {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<McTree");
    let mut sorted_keys : Vec<u32> = self.trackmap.keys().copied().collect();
    sorted_keys.sort();
    for k in sorted_keys {
      repr += &(format!("\n  Track Id: {} [pdg {}] -> NHits : {}",k,self.trackmap[&k].track.pdg, self.trackmap[&k].hits.len()));
    }
    repr += ">";
    write!(f, "{}", repr) 
  }
}

#[cfg(feature="pybindings")] 
#[pymethods] 
impl McTree {

  #[getter]
  fn get_trackids(&self) -> Vec<u32> {
    let mut sorted_keys : Vec<u32> = self.trackmap.keys().copied().collect();
    sorted_keys.sort();
    sorted_keys
  }

  fn get_track(&self, track_id : u32) -> Option<McTrack> {
    self.trackmap.get(&track_id).clone().cloned()
  }
}



#[cfg(feature="pybindings")]
pythonize!(McTree);
