import gaps_tof as gt
import pylab as p
import charmingbeauty as cb

# monkey patch the C++ API RBEvent
gt.RBEvent.calib = None



def _adc_plotter(axes, ev, ch, calib=None):
    if ch == 9:
        ax_j  = 4
        color = 'k'
        alpha = 1.0
        lw    = 1.2
    elif ch % 2 == 0:
        ax_j  = int ((ch - 2) / 2)
        color = 'r'
        alpha = 0.7
        lw    = 1.2
    else:
        ax_j  = int ((ch - 1) / 2)
        color = 'b'
        alpha = 0.7
        lw    = 1.2
    if calib is None:
        xs = ev.get_channel_adc(ch)
        ys = [k for k in range(1024)]
    else:
        xs = calib.voltages(ev, spike_cleaning=True)[ch - 1]
        ys = calib.nanoseconds(ev)[ch - 1]
    axes[ax_j].plot(xs,\
                    ys,\
                    color = color,\
                    alpha = alpha,\
                    lw    = lw)

def plot(self, calib=None):
    fig, axes = \
      p.subplots(5, 1, sharex=True, figsize=cb.layout.FIGSIZE_A4)# layout='constrained', sharex=True)
    fig.subplots_adjust(hspace=0)
    if calib is None:
        xlabel = '14bit-ADC bins'
        ylabel = 'Timing sample bins [2GS/s]'
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
        for ch in range(1,10):
            _adc_plotter(axes, self, ch)
    else:
        volts = calib.voltages(self, spike_cleaning = True)
        nanos = calib.nanoseconds(self)
        for ch in range(1,10):
            if ch % 2 == 0:
                color = "r"
            else:
                color = "b"
            if ch in [1,2]:
                axes[0].plot(nanos[ch-1], volts[ch-1], lw=1.2, color=color )
            if ch in [3,4]:
                axes[1].plot(nanos[ch-1], volts[ch-1], lw=1.2, color=color )
            if ch in [5,6]:
                axes[2].plot(nanos[ch-1], volts[ch-1], lw=1.2, color=color )
            if ch in [7,8]:
                axes[3].plot(nanos[ch-1], volts[ch-1], lw=1.2, color=color )
            if ch == 9:
                axes[4].plot(nanos[ch-1], volts[ch-1], lw=1.2, color="k")
    return fig


gt.RBEvent.plot = plot
RBEvent = gt.RBEvent
