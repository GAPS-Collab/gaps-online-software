/// Theming for liftof-tui

use ratatui::style::{
    Color,
    Style
};

/// Implementation of a color palette
#[derive(Debug, Copy, Clone)]
pub struct ColorSet {
  pub c0 : Color,
  pub c1 : Color,
  pub c2 : Color,
  pub c3 : Color,
  /// Used to highlight
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

/// Black and white color palette for highest contrast
pub const COLORSETBW : ColorSet = ColorSet::new(Color::Black, Color::White, 
                                                Color::White, Color::White,
                                                Color::Rgb(244,133,0));

/// A color palette designed for the OMILU branch
pub const COLORSETOMILU : ColorSet = ColorSet::new(Color::Rgb(5, 59, 80),
                                                   Color::Rgb(23, 107, 135),
                                                   Color::Rgb(100, 204, 197),
                                                   Color::Rgb(238, 238, 238),
                                                   Color::Rgb(225, 170, 116));

/// A color palette designed for the NIUHI brnach
pub const COLORSETNIUHI : ColorSet = ColorSet::new(Color::Rgb(0,41,170),
                                                   Color::Rgb(0,63,136),
                                                   Color::Rgb(0,80,157),
                                                   Color::Rgb(253,197,0),
                                                   Color::Rgb(255,213,0));


/// A color palette inspired by the recent Dune movie
pub const COLORSETDUNE : ColorSet = ColorSet::new(Color::Rgb(161,18,37),
                                                  Color::Rgb(223,135,53),
                                                  Color::Rgb(181,164,146),
                                                  Color::Rgb(225, 170, 116),
                                                  Color::Rgb(244,193,110));

/// A color palette inspired by Star Trek Lower Decks
pub const COLORSETLD    : ColorSet = ColorSet::new(Color::Rgb(255,68,0),
                                                   Color::Rgb(255, 170, 68),
                                                   Color::Rgb(255, 119, 0),
                                                   Color::Rgb(255, 204, 153),
                                                   Color::Rgb(255, 238, 204));

/// A color palette inspired by the original Matrix trilogy
pub const COLORSETMATRIX : ColorSet = ColorSet::new(Color::Rgb(2,2,4),
                                                    Color::Rgb(32,72,41),
                                                    Color::Rgb(34,180,85),
                                                    Color::Rgb(128,206,135),
                                                    Color::Rgb(156,229,161));

/// A color palette inspired by Bethesda's recent ARPG
pub const COLORSETSTARFIELD : ColorSet = ColorSet::new(Color::Rgb(48,76,122),
                                                       Color::Rgb(224,98,54),
                                                       Color::Rgb(215,166,75),
                                                       Color::Rgb(244,245,247),
                                                       Color::Rgb(199,33,56));

/// A color palette created from the colors of the GAPS logo
pub const COLORSETGAPS : ColorSet = ColorSet::new(Color::Rgb(27,51,88),
                                                  Color::Rgb(228,60,65),
                                                  Color::Rgb(132,203,187),
                                                  Color::Rgb(212,202,87),
                                                  Color::Rgb(227,76,68));

/// A pink color palette
pub const COLORSETPRINCESSPEACH : ColorSet = ColorSet::new(Color::Rgb(255,8,74),
                                                           Color::Rgb(252,52,104),
                                                           Color::Rgb(255,98,137),
                                                           Color::Rgb(255,147,172),
                                                           Color::Rgb(255,194,205));


/// A color theme, created from a color palette whcih 
/// allows to provide style variants for ui elements
#[derive(Debug, Copy, Clone)]
pub struct ColorTheme {
  pub bg0 : Color,
  pub bg1 : Color,
  pub fg0 : Color,
  pub fg1 : Color,
  pub hc  : Color,
}

impl ColorTheme {
  pub fn new() -> ColorTheme {
    ColorTheme {
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

