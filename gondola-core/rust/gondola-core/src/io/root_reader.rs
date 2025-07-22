//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use oxyroot::RootFile;

#[cfg(feature="pybindings")]
use pyo3::pyfunction;

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn read_example() {
  let s = "/srv/gaps/example-data/Run9125.gse5_241213_142800UTC_rec.root";
  let mut file = RootFile::open(s).expect("Can not open file");
  let tree = file.get_tree("TreeRec").expect("Can not get TTRee!");
  let one = tree.branch("Rec/hitseries_/hitseries_.energydep_").unwrap().as_iter::<i32>().expect("wrong type");
  one.for_each(|v| println!("v = {v}"));
}


//#[cfg_attr(feature="pybindings", pyfunction)]
//fn read_with_pyroot(path: &str, treename: &str, branchname: &str) -> PyResult<()> {
//    Python::with_gil(|py| {
//        let module = PyModule::from_code(py, r#"
//import ROOT
//
//def read_branch(filename, treename, branchname):
//    f = ROOT.TFile.Open(filename)
//    t = f.Get(treename)
//    for entry in t:
//        value = getattr(entry, branchname)
//        print(value)
//    f.Close()
//"#, "reader.py", "reader")?;
//
//        let read_branch = module.getattr("read_branch")?;
//        read_branch.call1((path, treename, branchname))?;
//
//        Ok(())
//    })
//}

