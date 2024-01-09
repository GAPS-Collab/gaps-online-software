/// Theming for liftof-tui

use ratatui::style::{
    Color,
    Style
};

#[derive(Debug, Copy, Clone)]
pub struct ColorSet {
  pub c0 : Color,
  pub c1 : Color,
  pub c2 : Color,
  pub c3 : Color,
  pub hc : Color,
}

impl ColorSet {
  pub const fn new(c0 : Color,
                   c1 : Color,
                   c2 : Color,
                   c3 : Color,
                   hc : Color) -> ColorSet {
    ColorSet {
      c0,
      c1,
      c2,
      c3,
      hc,
    }
  }
}

pub const COLORSETBW : ColorSet = ColorSet::new(Color::Black, Color::White, 
                                                Color::Black, Color::White,
                                                Color::Black);
pub const COLORSETOMILU : ColorSet = ColorSet::new(Color::Rgb(5, 59, 80),
                                                   Color::Rgb(23, 107, 135),
                                                   Color::Rgb(100, 204, 197),
                                                   Color::Rgb(238, 238, 238),
                                                   Color::Rgb(225, 170, 116));
pub const COLORSETNIUHI : ColorSet = ColorSet::new(Color::Rgb(0,41,170),
                                                   Color::Rgb(0,63,136),
                                                   Color::Rgb(0,80,157),
                                                   Color::Rgb(253,197,0),
                                                   Color::Rgb(255,213,0));
pub const COLORSETDUNE : ColorSet = ColorSet::new(Color::Rgb(223,135,53),
                                                  Color::Rgb(244,193,110),
                                                  Color::Rgb(181,164,146),
                                                  Color::Rgb(161,18,37),
                                                  Color::Rgb(225, 170, 116));
pub const COLORSETGMAPS : ColorSet = ColorSet::new(Color::Rgb(74,128,245),
                                                   Color::Rgb(155,191,244),
                                                   Color::Rgb(167,205,242),
                                                   Color::Rgb(187,218,164),
                                                   Color::Rgb(241,141,0));

pub const COLORSETLD    : ColorSet = ColorSet::new(Color::Rgb(255,68,0),
                                                   Color::Rgb(255, 170, 68),
                                                   Color::Rgb(255, 119, 0),
                                                   Color::Rgb(255, 204, 153),
                                                   Color::Rgb(255, 238, 204));

pub const COLORSETMATRIX : ColorSet = ColorSet::new(Color::Rgb(54,186,1),
                                                    Color::Rgb(0,154,34),
                                                    Color::Rgb(0,255,43),
                                                    Color::Rgb(0,154,34),
                                                    Color::Rgb(54,186,1));


#[derive(Debug, Copy, Clone)]
pub struct ColorTheme2 {
  pub bg0 : Color,
  pub bg1 : Color,
  pub fg0 : Color,
  pub fg1 : Color,
  pub hc  : Color,
}

impl ColorTheme2 {
  pub fn new() -> ColorTheme2 {
    ColorTheme2 {
      bg0 : Color::Black,
      bg1 : Color::White,
      fg0 : Color::Black,
      fg1 : Color::White,
      hc  : Color::White,
    }
  }

  pub fn update(&mut self, cs : &ColorSet) {
    self.bg0 = cs.c0;
    self.bg1 = cs.c1;
    self.fg0 = cs.c2;
    self.fg1 = cs.c3;
    self.hc  = cs.hc;
  }

  pub fn style(&self) -> Style {
    Style::default().bg(self.bg0).fg(self.fg1)
  }

  pub fn style_soft(&self) -> Style {
    Style::default().bg(self.bg1).fg(self.fg0)
  }

  pub fn highlight(&self) -> Style {
    Style::default().bg(self.hc).fg(self.fg1)
  }

  pub fn highlight_fg(&self) -> Style {
    Style::default().fg(self.hc)
  }

  pub fn background(&self) -> Style {
    Style::default().bg(self.bg0)
  }
}


/// Stylize everything with color themes
pub trait ColorTheme {
  /// color gradient from darkest (c0) to brightest (c1)
  const C0 : Color = Color::Black;
  const C1 : Color = Color::White;
  const C2 : Color = Color::Black;
  const C3 : Color = Color::White;
  
  /// color for highlights
  const HC : Color = Color::White;
  fn style(&self)        -> Style;

  fn style_soft(&self)   -> Style;

  fn highlight(&self)    -> Style;

  fn highlight_fg(&self) -> Style;

  fn background(&self)   -> Style;
}
