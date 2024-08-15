//! Basic analysis of taken data. Waveform visualisation
//!
//! WIP - currently this is in limbo!
//!
//!

use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::io::{
    BufReader,
    BufWriter
};
use std::collections::HashMap;


#[macro_use] extern crate log;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

use liftof_lib::{
    init_env_logger,
    LIFTOF_LOGO_SHOW
};

use rusttype::{Font, Scale};

use glob::glob;
//#[macro_use] extern crate printpdf;
extern crate printpdf;
use printpdf::*;
use printpdf::image::ImageTransform;
//use printpdf::image_crate;
//use plotters::prelude::*;
use plotters::style::IntoFont;
use plotters::prelude::Histogram as PHist;
use plotters::prelude::IntoTextStyle;
use plotters::prelude::FontDesc;
use plotters::prelude::FontStyle;
use plotters::prelude::TextStyle;
use plotters::chart::ChartBuilder;
use plotters::style::Color;
use plotters::prelude::WHITE;
use plotters::prelude::FontTransform;
use plotters::prelude::{
    BLUE,
    RED,
    BLACK,
};
use plotters::backend::BitMapBackend;
use plotters::drawing::IntoDrawingArea;
use ndhistogram::{
    ndhistogram,
    Histogram,
    //Hist1D,
};
use ndhistogram::axis::Uniform;

use std::io::Cursor;


use liftof_lib::settings::LiftofSettings;

use tof_dataclasses::database::{
    connect_to_db,
    ReadoutBoard
};

use tof_dataclasses::io::TofPacketReader;

use tof_dataclasses::packets::PacketType;
use tof_dataclasses::events::TofEvent;
use tof_dataclasses::serialization::Serialization;

#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Restrict the number of events we are running over
  #[arg(short, long, default_value_t = 0)]
  nevents: u64,
  /// Input data. Globbing (*) syntax supported. When using
  /// globbing syntax, please put the argument in "" quotation
  /// marks.
  #[arg(short, long, default_value_t = String::from(""))]
  input: String,
  /// Configuration of liftof-cc. Configure analysis engine,
  /// event builder and general settings.
  #[arg(short, long)]
  config: String,
}

fn main() {

  init_env_logger();
  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!("-----------------------------------------------");
  println!(" >> Welcome to liftof-analysis \u{1F680} \u{1F388} ");
  println!(" >> liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" >> for the GAPS experiment \u{1F496}");
  println!(" >> Offline analysis software to analyze .tof.gaps");
  println!(" >> data files");
  println!("-----------------------------------------------");

  // deal with command line arguments
  let args    = Args::parse();
  let config  : LiftofSettings;
  let config_file = args.config.clone();
  match LiftofSettings::from_toml(args.config) {
    Err(err) => {
      error!("CRITICAL! Unable to parse .toml settings file {}! {}", config_file, err);
      panic!("Unable to parse config file!");
    }
    Ok(_cfg) => {
      config = _cfg;
    }
  }
  let mut data_files = Vec::<String>::new();
  match glob(&args.input) {
    Ok(paths) => {
      let mut sorted_paths: Vec<PathBuf> = paths.filter_map(Result::ok).collect();
      sorted_paths.sort(); // Sorts the paths alphabetically

      for entry in sorted_paths {
        //println!("{:?}", path.display());
        data_files.push(entry.to_str().unwrap().to_owned());
      }
    },
    Err(e) => println!("Failed to read glob pattern: {}", e),
  }

  println!("-- Got following input data files");
  for k in data_files.iter() {
    println!("-- -- {}", k);
  }
  let db_path = config.db_path.clone();
  println!("-- Using data base at {}", config.db_path);
  println!("-- Will read calibration data from {}", config.calibration_dir);
  let mut conn              = connect_to_db(db_path).expect("Unable to establish a connection to the DB! CHeck db_path in the liftof settings (.toml) file!");
  // if this call does not go through, we might as well fail early.
  //let mut rb_list           = ReadoutBoard::all(&mut conn).expect("Unable to retrieve RB information! Unable to continue, check db_path in the liftof settings (.toml) file and DB integrity!");
  //let rb_ignorelist = config.rb_ignorelist.clone();
  //for k in 0..rb_ignorelist.len() {
  //  let bad_rb = rb_ignorelist[k];
  //  println!("=> We will INGORE RB {:02}, since it is being marked as IGNORE in the config file!", bad_rb);
  //  rb_list.retain(|x| x.rb_id != bad_rb);
  //}
  
  // loading the calibrations
  //println!("-- Loaded the following RBs");
  //for mut k in rb_list {
  //  k.calib_file_path = config.calibration_dir.clone();
  //  let _ = k.load_latest_calibration();
  //  println!("-- -- {}", k);
  //}
  //

  // write a nice report
  // Widescreen 21:9 format dimensions (e.g., 210mm x 90mm for simplicity)
  let (doc, page1, layer1) = PdfDocument::new("TOF Run Report", Mm(210.0), Mm(297.0), "Layer 1");
  let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Page 2, Layer 1");
  let mut fonts       = HashMap::<&str, IndirectFontRef>::new();
  let mut histo_fonts = HashMap::<&str, Vec<u8>>::new();
  // fallback Helvetica - for the ROOT lovers out there
  let fb_font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
  match File::open(Path::new("../../../resources/assets/fonts-minimal/Hack-Regular.ttf")) {
    Err(err) => {
      error!("Unable to find font file for Hack! Falling back to Helvetica!");
      fonts.insert("hack", fb_font.clone());
    }
    Ok(ff_hack) => {
      let font : IndirectFontRef = doc.add_external_font(ff_hack).unwrap(); 
      fonts.insert("hack", font);
    }
  }
  match File::open(Path::new("../../../resources/assets/fonts-minimal/Oswald-Regular.ttf")) {
    Err(err) => {
      error!("Unable to find font file for Oswald! Falling back to Helvetica!");
      fonts.insert("oswald", fb_font.clone());
    }
    Ok(ff_hack) => {
      let font : IndirectFontRef = doc.add_external_font(ff_hack).unwrap(); 
      fonts.insert("oswald", font);
    }
  }
  match File::open(Path::new("../../../resources/assets/fonts-minimal/SweetSansProRegular.ttf")) {
    Err(err) => {
      error!("Unable to find font file for SweetSansPro! Falling back to Helvetica!");
      fonts.insert("sweet", fb_font.clone());
    }
    Ok(ff_hack) => {
      let font : IndirectFontRef = doc.add_external_font(ff_hack).unwrap(); 
      fonts.insert("sweet", font);
    }
  }
  match File::open(Path::new("../../../resources/assets/fonts-minimal/OpenSans-Regular.ttf")) {
    Err(err) => {
      error!("Unable to find font file for OpenSans! Falling back to Helvetica!");
      fonts.insert("sans", fb_font.clone());
    }
    Ok(ff_hack) => {
      let font : IndirectFontRef = doc.add_external_font(ff_hack).unwrap(); 
      fonts.insert("sans", font);
    }
  }
  
  let layer1 = doc.get_page(page1).get_layer(layer1);
  layer1.use_text("GAPS TOF run report", 20.0, Mm(120.0), Mm(270.0), fonts.get("sans").unwrap());
  let mut image_file = File::open("../../../resources/assets/GAPSLOGO_2023_small.bmp").unwrap();
  let image = Image::try_from(image_crate::codecs::bmp::BmpDecoder::new(&mut image_file).unwrap()).unwrap();
  let mut transform = ImageTransform::default();
  transform.translate_x = Some(Mm(10.0));
  transform.translate_y = Some(Mm(265.0));
  transform.scale_x     = Some(0.8);
  transform.scale_y     = Some(0.8);
  image.add_to_layer(layer1.clone(), transform);
  let points1 = vec![(Point::new(Mm(10.0),  Mm(260.0)), false),
                     (Point::new(Mm(200.0), Mm(260.0)), false),
                     (Point::new(Mm(200.0), Mm(259.5)), false),
                     (Point::new(Mm(10.0), Mm(259.5)), false),

  ];
  // Is the shape stroked? Is the shape closed? Is the shape filled?
  let line1 = Line {
      points: points1,
      is_closed: true,
      has_fill: true,
      has_stroke: true,
      is_clipping_path: false,
  };
  //let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
  //layer1.set_fill_color(fill_color);
  layer1.add_shape(line1);
  layer1.use_text("Datafiles", 14.0, Mm(10.0), Mm(250.0), fonts.get("sweet").unwrap());
  let mut ypos = 245.0;
  let mut lw   = 4.0;
  for f in &data_files {
    let text = format!("-- {}", f);
    layer1.use_text(&text, 10.0, Mm(10.0), Mm(ypos), fonts.get("hack").unwrap());
    ypos -= lw;
  }
  //let layer2 = doc.get_page(page2).get_layer(layer2);
  //layer2.use_text("Hello from Page 2!", 20.0, Mm(20.0), Mm(70.0), fonts.get("oswald").unwrap());



  // setup progress bar
  //let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
  //let bar_label  = String::from("Acquiring RB calibration data");
  //let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
  //let bar = ProgressBar::new(rb_list.len() as u64); 
  //bar.set_position(0);
  //bar.set_message (bar_label);
  //bar.set_prefix  ("\u{2699}\u{1F4D0}");
  //bar.set_style   (bar_style);
  let mut n_events       = 0u64;
  let mut last_event_id  = 0u32;
  let mut skipped_events = 0u64;
  let mut n_unpack_error = 0u64;
  let mut event_ids      = Vec::<u32>::new();
  let mut n_trigger_hits = 0u64;
  let mut n_rb_channels  = 0u64;

  // paddle histogram
  let bins_pid   = Uniform::new(160,1.0, 160.0);
  let mut pid_histo  = ndhistogram!(bins_pid);

  for f in data_files {
    println!("-- -- Reading from file {}", f);
    let reader = TofPacketReader::new(f);
    for pack in reader {
      //println!("-- {}", pack);
      match pack.packet_type {
        PacketType::TofEvent => {
          n_events += 1;
          match TofEvent::from_tofpacket(&pack) {
            Err(err) => {
              error!("Can't unpack event! {err}");
              n_unpack_error += 1;
            },
            Ok(event) => {
              let event_id = event.header.event_id;
              let nhit_exp = event.mt_event.get_trigger_hits();
              n_trigger_hits += nhit_exp.len() as u64;
              for rbevent in event.rb_events {
                let hits = rbevent.header.get_channels().len();
                for h in rbevent.hits {
                  pid_histo.fill(&(h.paddle_id as f32));
                }
                n_rb_channels += hits as u64;
              }
              event_ids.push(event_id);
            }
          }
        },
        _ => ()
      }
    }
  }
  // missing event analysis
  event_ids.sort();
  if event_ids.len() > 0 {
    last_event_id = event_ids[0];
    for k in 1..event_ids.len() {
      let event_id = event_ids[k];
      let delta = event_id as u64 - last_event_id as u64;
      if delta > 1 {
        skipped_events += delta - 1;
      }
      last_event_id = event_id;
    }
    //  println!("event id {}, last event id {}", event_id, last_event_id);
    //}
  }
  println!("-- Read {} TofEvents!", n_events);
  if n_unpack_error > 0 {
    println!("-- Experienced {} unpack errors!", n_unpack_error);
  }
  let skipped_event_str =  format!("-- We skipped {} event ids or {:.2}%", skipped_events, 100.0 * skipped_events as f64 / n_events as f64);
  println!("{}", skipped_event_str);

  let av_trigger_hits = n_trigger_hits as f64 / n_events as f64;
  let av_rb_channels  = n_rb_channels  as f64 / n_events as f64;
  let lost_hits_perc  = 100.0 - (100.0 * av_rb_channels / av_trigger_hits);
  println!("-- We saw {:.3} hits/event (MasterTrigger)", av_trigger_hits);
  println!("-- We saw {:.3} hits/event (ReadoutBoards)", av_rb_channels);
  println!("-- We missed {:.3} % of triggered hits!", lost_hits_perc);
  
  let av_trig_hits_str   = format!("-- We saw {:.3} hits/event (MasterTrigger)", av_trigger_hits);
  let av_rb_channels_str = format!("-- We saw {:.3} hits/event (ReadoutBoards)", av_rb_channels);
  let lost_hits_perc_str = format!("-- We missed {:.3} % of triggered hits!", lost_hits_perc);
  ypos -= 5.0;
  layer1.use_text("Missing event analysis:", 14.0, Mm(10.0), Mm(ypos), fonts.get("sweet").unwrap());
  ypos -= lw;
  layer1.use_text(&skipped_event_str, 10.0, Mm(10.0), Mm(ypos), fonts.get("hack").unwrap());
  ypos -= lw;
  layer1.use_text(&av_trig_hits_str, 10.0, Mm(10.0), Mm(ypos), fonts.get("hack").unwrap());
  ypos -= lw;
  layer1.use_text(&av_rb_channels_str, 10.0, Mm(10.0), Mm(ypos), fonts.get("hack").unwrap());
  ypos -= lw;
  layer1.use_text(&lost_hits_perc_str, 10.0, Mm(10.0), Mm(ypos), fonts.get("hack").unwrap());
  ypos -= lw;

  let mut piddata = Vec::<(i32,usize)>::new();
  let mut maxbin_val = 0usize;
  for bin in pid_histo.iter() {
    if *bin.value as usize > maxbin_val {
      maxbin_val = *bin.value as usize;
    }
    piddata.push((bin.index as i32, *bin.value as usize)); 
  }
  maxbin_val = maxbin_val + (maxbin_val as f64 * 0.1).floor() as usize;
  // create histograms
  let root_drawing_area = BitMapBackend::new("histogram.bmp", (1024, 768)).into_drawing_area();
  root_drawing_area.fill(&WHITE);

  // Manually draw the chart title
  let font_data = std::fs::read("../../../resources/assets/fonts-minimal/SweetSansProRegular.ttf").unwrap();
  let font = Font::try_from_vec(font_data).ok_or("Error loading font").unwrap();
  let scale = Scale::uniform(50.0);
  //let font_transform = FontTransform::new(font, scale);
  //let font_desc = FontDesc::new(font, 20.0, FontStyle::Normal);
  let font = ("Arial", 20).into_font();

  // Note: Adjust the text color, font, and position as needed
  //let style = TextStyle::from(&font).color(&BLACK);
  //let font: FontDesc = FontDesc::try_from_file(Path::new("../../../resources/assets/fonts-minimal/SweetSansProRegular.ttf"), 50.0);
  //root_drawing_area.draw_text(
  //    "My Custom Title",
  //    &TextStyle::from(&font_desc),
  //    (950, 50) // Adjust these coordinates as needed
  //);
  let mut chart = ChartBuilder::on(&root_drawing_area)
    .caption("Paddle occupancy", ("sans-serif", 20))
    .x_label_area_size(30)
    .y_label_area_size(30)
    .margin(5)
    .build_cartesian_2d(0..165, 0..maxbin_val).unwrap(); // Adjust the y-axis range according to your data

  chart.configure_mesh()
    .x_desc("Paddle ID") // Set the x-axis label
    .y_desc("Occupancy (hits)") // Set the y-axis label
    .draw().unwrap();
  chart.draw_series(
    PHist::vertical(&chart)
      .style(BLUE.mix(0.9).filled())
      //.data(piddata.iter().map(|&(x, y)| (x, y))),
      .data(piddata)
  ).unwrap();
  // Generate some example data for the histogram
  // This is where you would put your actual data
  match root_drawing_area.present() {
    Ok(_) => (),
    Err(_err) =>(),
  }
  println!("Histogram saved to 'histogram.bmp'");



  // Save the PDF
  doc.save(&mut BufWriter::new(File::create("widescreen.pdf").unwrap())).unwrap();
} // end main

