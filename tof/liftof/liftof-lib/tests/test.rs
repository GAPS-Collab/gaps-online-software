use liftof_lib::LiftofSettings;

#[test]
fn write_config_file() {
  let settings = LiftofSettings::new();
  println!("{}", settings);
  settings.to_toml(String::from("liftof-config-test.toml"));
}

#[test]
fn read_config_file() {
  let _settings = LiftofSettings::from_toml(String::from("liftof-config-test.toml"));
}
