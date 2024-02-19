import gaps_tof as gt
import pylab as p
import numpy as np
try:
    import charmingbeauty as cb
    FIGSIZE=cb.layout.FIGSIZE_A4  
except ImportError as e:
    print(f"Can't find charmingbeauty for nice looking plots! {e}")
    GOLDEN_RATIO = (1 + np.sqrt(5))/2.
    WIDTH_A4 = 5.78851 # inch
    FIGSIZE_A4_LANDSCAPE = (WIDTH_A4, WIDTH_A4/GOLDEN_RATIO)
    FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT = (WIDTH_A4, (WIDTH_A4/(2*GOLDEN_RATIO)))
    FIGSIZE_A4 = (WIDTH_A4, (WIDTH_A4/GOLDEN_RATIO) + WIDTH_A4)
    FIGSIZE=FIGSIZE_A4

# monkey patch the C++ API RBEvent
gt.RBEvent.calib = None

def _adc_plotter(axes, ev, ch, calib=None, plot_stop_cell=False, skip_first_bins = 0):
    """
    Plot the raw adc values/time vs voltages for all channels for all events

    # Arguments

      * axes
      * ev
      * ch
      * calib
      * plot_stop_cell
      * skip_first_bins
    """

    if ch == 9:
        ax_j  = 4
        color = 'k'
        alpha = 1.0
        lw    = 1.2
    elif ch % 2 == 0:
        ax_j  = int ((ch - 2) / 2)
        color = 'tab:red'
        alpha = 0.9
        lw    = 0.9
    else:
        ax_j  = int ((ch - 1) / 2)
        color = 'tab:blue'
        alpha = 0.9
        lw    = 0.9
    if calib is None:
        xs = [k for k in range(1024)]
        ys = ev.get_channel_adc(ch)
    else:
        xs = calib.nanoseconds(ev)[ch - 1]
        ys = calib.voltages(ev, spike_cleaning=True)[ch - 1]
    axes[ax_j].plot(xs[skip_first_bins:],\
            ys[skip_first_bins:],\
                    color = color,\
                    alpha = alpha,\
                    lw    = lw)
    if plot_stop_cell:
        axes[ax_j].vlines(ev.header.stop_cell, *axes[ax_j].get_xlim(),lw=0.9, ls='dashed', color='gray')

def plot(self,\
         calib : gt.RBCalibration = None,
         spike_cleaning = True,
         plot_stop_cell = False,
         skip_first_bins = 0):
    """
    Plot (un)calibrated waveforms of this event.
    All channels.

    # Keyword Args:
      * calib           : RBCalibration for this board
      * remove_spikes   : apply DRS4 spike cleaning routine
      * plot_stop_cell  : 
      * skip_first_bins : remove the first n bins from the plot 
                          (in case there are big spikes in the
                           beginning)
    """

    fig, axes = \
      p.subplots(5, 1, sharex=True, figsize=cb.layout.FIGSIZE_A4)# layout='constrained', sharex=True)
    fig.subplots_adjust(hspace=0)
    if calib is None:
        xlabel = 'Timing sample bins [2GS/s]'
        ylabel = '14bit-ADC bins'
        xpos   = 0.5
    else:
        xlabel = 'nanoseconds'
        ylabel = 'milli Volts'
        xpos   = 0.7
    fig.text(xpos, 0.05,\
            xlabel,\
            transform=fig.transFigure,\
            size=14)
    fig.text(-0.05, 0.7, ylabel,\
            rotation=90,\
            transform=fig.transFigure,\
            size=14)
    fig.text(0.5, 0.9,\
            f'RB {self.header.rb_id}, ev id {self.header.event_id}',\
             transform=fig.transFigure,\
             size=18) 
    for k in axes:
        k.spines['top'].set_visible(True)
        k.spines['right'].set_visible(True)
        k.grid(True)
    if calib is None:
        print ("No calibration given! Will plot adc values!")
        axes[0].set_xlabel("")
        for ch in self.header.get_channels():
        #for ch in range(1,10):
            ch += 1
            _adc_plotter(axes, self, ch, plot_stop_cell = plot_stop_cell, skip_first_bins = skip_first_bins)
    else:
        volts = calib.voltages(self, spike_cleaning = spike_cleaning)
        nanos = calib.nanoseconds(self)
        for ch in self.header.get_channels():
            ch += 1
        #for ch in range(1,10):
            if ch % 2 == 0:
                color = "r"
            else:
                color = "b"
            if ch in [1,2]:
                axes[0].plot(nanos[ch-1][skip_first_bins:], volts[ch-1][skip_first_bins:], lw=1.2, color=color )
                if plot_stop_cell:
                    stop_cell_time = nanos[ch-1][self.header.stop_cell]
                    axes[0].vlines(stop_cell_time, *axes[0].get_xlim(), lw=0.9, ls='dashed', color='gray')
            if ch in [3,4]:
                axes[1].plot(nanos[ch-1][skip_first_bins:], volts[ch-1][skip_first_bins:], lw=1.2, color=color )
                if plot_stop_cell:
                    stop_cell_time = nanos[ch-1][self.header.stop_cell]
                    axes[1].vlines(stop_cell_time, *axes[0].get_xlim(), lw=0.9, ls='dashed', color='gray')
            if ch in [5,6]:
                axes[2].plot(nanos[ch-1][skip_first_bins:], volts[ch-1][skip_first_bins:], lw=1.2, color=color )
                if plot_stop_cell:
                    stop_cell_time = nanos[ch-1][self.header.stop_cell]
                    axes[2].vlines(stop_cell_time, *axes[0].get_xlim(), lw=0.9, ls='dashed', color='gray')
                    pass
            if ch in [7,8]:
                axes[3].plot(nanos[ch-1][skip_first_bins:], volts[ch-1][skip_first_bins:], lw=1.2, color=color )
                if plot_stop_cell:
                    stop_cell_time = nanos[ch-1][self.header.stop_cell]
                    axes[3].vlines(stop_cell_time, *axes[0].get_xlim(), lw=0.9, ls='dashed', color='gray')
            if ch == 9:
                axes[4].plot(nanos[ch-1][skip_first_bins:], volts[ch-1][skip_first_bins:], lw=1.2, color="k")
                if plot_stop_cell:
                    stop_cell_time = nanos[ch-1][self.header.stop_cell]
                    axes[4].vlines(stop_cell_time, *axes[0].get_xlim(), lw=0.9, ls='dashed', color='gray')
    return fig


gt.RBEvent.plot = plot
RBEvent = gt.RBEvent
