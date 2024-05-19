//! Heartbeats for different treads
//!
//! Metadata for debugging
//!

struct HeartBeatDataSink {

  /// mission elapsed time in seconds
  pub met                : u64;
  pub n_packets_sent     : u64;
  pub n_packets_incoming : u64;
  /// bytes written to disk
  pub n_bytes_written    : u64;
  /// event id check - missing event ids
  pub n_evid_missing     : u64;
  /// event id check - chunksize
  pub n_evid_chunksize   : u64;
}


//println!("  {:<75}", ">> == == == == == == DATA SINK HEARTBEAT  == == == == == == <<".bright_cyan().bold());
//      println!("  {:<75} <<", format!(">> ==> Sent {} TofPackets! (packet rate {:.2}/s)", n_pack_sent ,packet_rate).bright_cyan());
//      println!("  {:<75} <<", format!(">> ==> Incoming cb channel len {}", incoming.len()).bright_cyan());
//      println!("  {:<75} <<", format!(">> ==> Writing events to disk: {} packets written, data write rate {:.2} MB/sec", n_pack_write_disk, bytes_sec_disk/(1e6*met_time_secs as f64)).bright_purple());
//      println!("  {:<75} <<", format!(">> ==> Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", evid_missing, evid_check_len, 100.0*(evid_missing as f64/evid_check_len as f64)).bright_purple());
//
//      println!("  {:<75}", ">> == == == == == == == == == == == == == == == == == == == <<".bright_cyan().bold());
//      timer = Instant::now();
//}
