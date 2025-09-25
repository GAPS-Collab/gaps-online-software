//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

diesel::table! {
  auth_group (id) {
      id -> Integer,
      name -> Text,
  }
}

diesel::table! {
  auth_group_permissions (id) {
      id -> Integer,
      group_id -> Integer,
      permission_id -> Integer,
  }
}

diesel::table! {
  auth_permission (id) {
      id -> Integer,
      content_type_id -> Integer,
      codename -> Text,
      name -> Text,
  }
}

diesel::table! {
  auth_user (id) {
      id -> Integer,
      password -> Text,
      last_login -> Nullable<Timestamp>,
      is_superuser -> Bool,
      username -> Text,
      last_name -> Text,
      email -> Text,
      is_staff -> Bool,
      is_active -> Bool,
      date_joined -> Timestamp,
      first_name -> Text,
  }
}

diesel::table! {
  auth_user_groups (id) {
      id -> Integer,
      user_id -> Integer,
      group_id -> Integer,
  }
}

diesel::table! {
  auth_user_user_permissions (id) {
      id -> Integer,
      user_id -> Integer,
      permission_id -> Integer,
  }
}

diesel::table! {
  django_admin_log (id) {
      id -> Integer,
      object_id -> Nullable<Text>,
      object_repr -> Text,
      action_flag -> SmallInt,
      change_message -> Text,
      content_type_id -> Nullable<Integer>,
      user_id -> Integer,
      action_time -> Timestamp,
  }
}

diesel::table! {
  django_content_type (id) {
      id -> Integer,
      app_label -> Text,
      model -> Text,
  }
}

diesel::table! {
  django_migrations (id) {
      id -> Integer,
      app -> Text,
      name -> Text,
      applied -> Timestamp,
  }
}

diesel::table! {
  django_session (session_key) {
      session_key -> Text,
      session_data -> Text,
      expire_date -> Timestamp,
  }
}

diesel::table! {
  tof_db_dsicard (dsi_id) {
      dsi_id -> SmallInt,
      j1_rat_id -> Nullable<SmallInt>,
      j2_rat_id -> Nullable<SmallInt>,
      j3_rat_id -> Nullable<SmallInt>,
      j4_rat_id -> Nullable<SmallInt>,
      j5_rat_id -> Nullable<SmallInt>,
  }
}

diesel::table! {
  tof_db_run (run_id) {
      run_id       -> BigInt,
      runtime_secs -> Nullable<BigInt>,
      calib_before -> Nullable<Bool>,
      shifter      -> Nullable<SmallInt>,
      run_type     -> Nullable<SmallInt>,
      run_path     -> Nullable<Text>,
  }
}

diesel::table! {
  tof_db_trackerstrip (strip_id) {
    strip_id            -> Integer,
    layer               -> Integer, 
    row                 -> Integer, 
    module              -> Integer, 
    channel             -> Integer,  
    global_pos_x_l0     -> Float,
    global_pos_y_l0     -> Float,
    global_pos_z_l0     -> Float,
    global_pos_x_det_l0 -> Float,
    global_pos_y_det_l0 -> Float,
    global_pos_z_det_l0 -> Float,
    principal_x         -> Float,
    principal_y         -> Float,
    principal_z         -> Float,
    volume_id           -> BigInt,
  }
}

diesel::table! {
  tof_db_trackerstripmask (data_id) {
    data_id             -> Integer,
    strip_id            -> Integer,
    volume_id           -> BigInt, 
    utc_timestamp_start -> BigInt,
    utc_timestamp_stop  -> BigInt,
    name                -> Nullable<Text>,
    active              -> Bool
  }
}

diesel::table! { 
  tof_db_trackerstrippedestal ( data_id ) {
    data_id             -> Integer,
    strip_id            -> Integer,
    volume_id           -> BigInt,
    utc_timestamp_start -> BigInt,
    utc_timestamp_stop  -> BigInt,
    name                -> Nullable<Text>,
    pedestal_mean       -> Float,
    pedestal_sigma      -> Float,
    is_mean_value       -> Bool,
  }
}

diesel::table! { 
  tof_db_trackerstriptransferfunction ( data_id ) {
    data_id             -> Integer,
    strip_id            -> Integer,
    volume_id           -> BigInt,
    utc_timestamp_start -> BigInt,
    utc_timestamp_stop  -> BigInt,
    name                -> Nullable<Text>, 
    // poly a (square)
    pol_a2_0       -> Float, 
    pol_a2_1       -> Float,    
    pol_a2_2       -> Float, 
    // poly b (cube)
    pol_b3_0       -> Float, 
    pol_b3_1       -> Float, 
    pol_b3_2       -> Float, 
    pol_b3_3       -> Float, 
    // poly c (cube)
    pol_c3_0       -> Float, 
    pol_c3_1       -> Float, 
    pol_c3_2       -> Float, 
    pol_c3_3       -> Float, 
    // poly d (cube)
    pol_d3_0       -> Float,     
    pol_d3_1       -> Float, 
    pol_d3_2       -> Float, 
    pol_d3_3       -> Float, 
  }
}

diesel::table! { 
  tof_db_trackerstripcmnnoise ( data_id ) {
    data_id             -> Integer,
    strip_id             -> Integer,
    volume_id            -> BigInt,
    utc_timestamp_start  -> BigInt,
    utc_timestamp_stop   -> BigInt,
    name                 -> Nullable<Text>, 
    gain                 -> Float,
    pulse_chn            -> Integer,
    pulse_avg            -> Float,
    gain_is_mean         -> Bool,
    pulse_is_mean        -> Bool
  }
}

diesel::table! {
  tof_db_localtriggerboard (board_id) {
      board_id -> SmallInt,
      dsi -> Nullable<SmallInt>,
      j -> Nullable<SmallInt>,
      rat -> Nullable<SmallInt>,
      ltb_id -> Nullable<SmallInt>,
      cable_len -> Float,
      paddle1_id -> Nullable<SmallInt>,
      paddle2_id -> Nullable<SmallInt>,
      paddle3_id -> Nullable<SmallInt>,
      paddle4_id -> Nullable<SmallInt>,
      paddle5_id -> Nullable<SmallInt>,
      paddle6_id -> Nullable<SmallInt>,
      paddle7_id -> Nullable<SmallInt>,
      paddle8_id -> Nullable<SmallInt>,
  }
}

diesel::table! {
  tof_db_mtbchannel (mtb_ch) {
      mtb_ch -> BigInt,
      dsi -> Nullable<SmallInt>,
      j -> Nullable<SmallInt>,
      ltb_id -> Nullable<SmallInt>,
      ltb_ch -> Nullable<SmallInt>,
      rb_id -> Nullable<SmallInt>,
      rb_ch -> Nullable<SmallInt>,
      mtb_link_id -> Nullable<SmallInt>,
      paddle_id -> Nullable<SmallInt>,
      paddle_isA -> Nullable<Bool>,
      hg_ch -> Nullable<SmallInt>,
      lg_ch -> Nullable<SmallInt>,
  }
}

diesel::table! {
  tof_db_paddle (paddle_id) {
      paddle_id          -> SmallInt,
      volume_id          -> BigInt,
      panel_id           -> SmallInt,
      mtb_link_id        -> SmallInt,
      rb_id              -> SmallInt,
      rb_chA             -> SmallInt,
      rb_chB             -> SmallInt,
      ltb_id             -> SmallInt,
      ltb_chA            -> SmallInt,
      ltb_chB            -> SmallInt,
      pb_id              -> SmallInt,
      pb_chA             -> SmallInt,
      pb_chB             -> SmallInt,
      cable_len          -> Float,
      dsi                -> SmallInt,
      j_rb               -> SmallInt,
      j_ltb              -> SmallInt,
      height             -> Float,
      width              -> Float,
      length             -> Float,
      normal_x           -> Float,
      normal_y           -> Float,
      normal_z           -> Float,
      global_pos_x_l0    -> Float,
      global_pos_y_l0    -> Float,
      global_pos_z_l0    -> Float,
      global_pos_x_l0_A  -> Float,
      global_pos_y_l0_A  -> Float,
      global_pos_z_l0_A  -> Float,
      global_pos_x_l0_B  -> Float,
      global_pos_y_l0_B  -> Float,
      global_pos_z_l0_B  -> Float,
      coax_cable_time    -> Float,
      harting_cable_time -> Float,
  }
}

diesel::table! {
  tof_db_panel (panel_id) {
      panel_id -> SmallInt,
      description -> Text,
      normal_x -> SmallInt,
      normal_y -> SmallInt,
      normal_z -> SmallInt,
      dw_paddle -> Nullable<SmallInt>,
      dh_paddle -> Nullable<SmallInt>,
      paddle0_id -> Nullable<SmallInt>,
      paddle1_id -> Nullable<SmallInt>,
      paddle10_id -> Nullable<SmallInt>,
      paddle11_id -> Nullable<SmallInt>,
      paddle2_id -> Nullable<SmallInt>,
      paddle3_id -> Nullable<SmallInt>,
      paddle4_id -> Nullable<SmallInt>,
      paddle5_id -> Nullable<SmallInt>,
      paddle6_id -> Nullable<SmallInt>,
      paddle7_id -> Nullable<SmallInt>,
      paddle8_id -> Nullable<SmallInt>,
      paddle9_id -> Nullable<SmallInt>,
  }
}

diesel::table! {
  tof_db_rat (rat_id) {
      rat_id -> SmallInt,
      pb_id -> SmallInt,
      rb1_id -> SmallInt,
      rb2_id -> SmallInt,
      ltb_id -> SmallInt,
      ltb_harting_cable_length -> SmallInt,
  }
}

diesel::table! {
  tof_db_readoutboard (rb_id) {
      rb_id        -> SmallInt,
      dsi          -> SmallInt,
      j            -> SmallInt,
      mtb_link_id  -> SmallInt,
      paddle12_chA -> Nullable<SmallInt>,
      paddle34_chA -> Nullable<SmallInt>,
      paddle56_chA -> Nullable<SmallInt>,
      paddle78_chA -> Nullable<SmallInt>,
      paddle12_id  -> Nullable<SmallInt>,
      paddle34_id  -> Nullable<SmallInt>,
      paddle56_id  -> Nullable<SmallInt>,
      paddle78_id  -> Nullable<SmallInt>,
  }
}


diesel::joinable!(auth_group_permissions -> auth_group (group_id));
diesel::joinable!(auth_group_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_permission -> django_content_type (content_type_id));
diesel::joinable!(auth_user_groups -> auth_group (group_id));
diesel::joinable!(auth_user_groups -> auth_user (user_id));
diesel::joinable!(auth_user_user_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_user_user_permissions -> auth_user (user_id));
diesel::joinable!(django_admin_log -> auth_user (user_id));
diesel::joinable!(django_admin_log -> django_content_type (content_type_id));

diesel::allow_tables_to_appear_in_same_query!(
  auth_group,
  auth_group_permissions,
  auth_permission,
  auth_user,
  auth_user_groups,
  auth_user_user_permissions,
  django_admin_log,
  django_content_type,
  django_migrations,
  django_session,
  tof_db_dsicard,
  tof_db_run,
  tof_db_localtriggerboard,
  tof_db_mtbchannel,
  tof_db_paddle,
  tof_db_panel,
  tof_db_rat,
  tof_db_readoutboard,
  tof_db_trackerstrip,
);
