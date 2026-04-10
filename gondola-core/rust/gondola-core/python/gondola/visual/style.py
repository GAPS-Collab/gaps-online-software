"""
Consistent plotstyles, e.g. to be used in gander, the online/live 
monitoring tool of gaps-online-software
"""

import matplotlib
import matplotlib.pyplot as plt

import charmingbeauty.layout as lo
import charmingbeauty as cb

import HErmes as he
import dashi as d
d.visual()

#--------------------------------------------

def gander_scatter_plot(xs, ys,
                        xlabel    : str,
                        ylabel    : str,
                        title     : str,
                        figsize   = lo.FIGSIZE_A4_LANDSCAPE,
                        **kwargs) -> matplotlib.figure.Figure:
    if not kwargs:
        kwargs = {'color'  : 'w',\
                  'alpha'  : 0.4,\
                  'marker' : '+'}
    #kwargs.update({'label' : label})
    fig = plt.figure(figsize=figsize)
    ax = fig.gca()
    #ax.legend(loc='upper right', frameon=False)
    ax.scatter(xs,ys, **kwargs)
    #ax.set_ylim(bottom=0)
    
    ax.set_xlabel(xlabel, loc='right')
    ax.set_ylabel(ylabel, loc='top')#, rotation=0)
    ax.set_title(title, loc='right')
    #if log:
    #    ax.set_yscale('symlog')
    cb.visual.adjust_minor_ticks(ax, which='x')
    return fig

#################################################

def gander_line_plot(xs, ys,
                     xlabel    : str,
                     ylabel    : str,
                     title     : str,
                     figsize   = lo.FIGSIZE_A4_LANDSCAPE,
                     **kwargs) -> matplotlib.figure.Figure:
    if not kwargs:
        kwargs = {'color'  : 'w',\
                  'alpha'  : 0.4,\
                  'lw'     : 0.9}
    #kwargs.update({'label' : label})
    fig = plt.figure(figsize=figsize)
    ax = fig.gca()
    #ax.legend(loc='upper right', frameon=False)
    ax.plot(xs,ys, **kwargs)
    #ax.set_ylim(bottom=0)
    
    ax.set_xlabel(xlabel, loc='right')
    ax.set_ylabel(ylabel, loc='top')
    ax.set_title(title, loc='right')
    #if log:
    #    ax.set_yscale('symlog')
    cb.visual.adjust_minor_ticks(ax, which='x')
    return fig

#################################################

def gander_plot(h          : d.histogram.hist1d,
                xlabel     : str,
                title      : str,
                figsize    = lo.FIGSIZE_A4_LANDSCAPE,
                gauss_fit  = False,
                use_gaps_style = False,
                landau_fit = False,
                log        = False,
                **kwargs) -> matplotlib.figure.Figure:
    """
    A plot with a default style for the 
    streamlit app

    # Arguemtns:

    # Keyword Arguments:
        **kwargs : to be passed to d.hist1d.line
    """
    if not kwargs:
        kwargs = {'color'  : 'w',\
                  'filled' : True,\
                  'alpha'  : 0.4,\
                  'lw'     : 0.9}
    #kwargs.update({'label' : label})
    fig = plt.figure(figsize=figsize)
    ax = fig.gca()
    h.line(**kwargs)
    #ax.legend(loc='upper right', frameon=False)
    text_xpos = 0.7
    if use_gaps_style:
        text_xpos = 0.15
    ax.text(text_xpos,0.8, f'N = {h.stats.nentries:.2e}', transform=fig.transFigure)
    if gauss_fit:
        def gauss(x, mu, sigma, amp):
            return he.fitting.gauss(x, mu, sigma)*amp
        model = he.fitting.model.Model(gauss)
        model.add_data(h.bincontent, xs=h.bincenters, data_errs=h.binwidths)
        model.startparams = (h.stats.mean,h.stats.std,max(h.bincontent))
        model.fit_to_data(silent=True)
        #model.fit_to_data(limits=[(-1,1),(0.001, 1), (1, max(panel[k].bincontent.sum(),1000))])
        print (f'-> best fit pars {model.best_fit_params}')
        ax.plot(model.xs, gauss(model.xs, *model.best_fit_params), color=kwargs['color'], lw=2)
        ax.text(text_xpos,0.75, fr'$\mu$   = {model.best_fit_params[0]:.2f}', transform=fig.transFigure)
        ax.text(text_xpos,0.7, fr'$\sigma$ = {model.best_fit_params[1]:.2f}', transform=fig.transFigure)
    if landau_fit:
        def Landau(xs, scale, mu, eta):
            return scale*scipy.stats.moyal.pdf(xs, loc=mu, scale=eta )

        model = he.fitting.model.Model(Landau)
        spectral = h.bincenters, h.bincontent
        model.startparams = (max(spectral[1]), 1 ,1111.15)
        model.add_data(h.bincontent, xs=h.bincenters, data_errs=h.binwidths, create_distribution=False)
        model.fit_to_data(silent=True)
        ax.plot(model.xs, Landau(model.xs, *model.best_fit_params), color=kwargs['color'], lw=2)
        ax.text(text_xpos,0.75, f'$\mu$   = {model.best_fit_params[1]:.2f}', transform=fig.transFigure)
        #ax.text(text_xpos,0.7, f'$\sigma$ = {model.best_fit_params[1]:.2f}', transform=fig.transFigure)

    ax.set_title(title, loc='right')
    if landau_fit:
        ax.set_ylim(bottom=0.1)
    else:
        ax.set_ylim(bottom=0)
     
    ax.set_xlabel(xlabel, loc='right')
    if log:
        if landau_fit:
            ax.set_yscale('log')
        else:
            ax.set_yscale('symlog')
    cb.visual.adjust_minor_ticks(ax, which='x')
    return fig

#################################################

def gander_multi_plot(h      : list[d.histogram.hist1d],
                      xlabel : str, 
                      title  : str,
                      labels : list[str],
                      kwargs : list[dict],
                      use_gaps_theme = False,
                      log            = False) -> matplotlib.figure.Figure:
    """
    A plot with a default style for the 
    streamlit app

    # Arguemtns:

        * kwargs         : to be passed to d.hist1d.line
    
    # Keyword Arguments:
        * use_gaps_theme : overide styling settings for the histograms
                           with the "official" GAPS style (K.Perez, S.Vickers)
    """
    if not kwargs:
        kwargs = [{'color'  : 'w',\
                  'filled' : True,\
                  'alpha'  : 0.4,\
                  'lw'     : 0.9} for k in range(len(h))]
    if use_gaps_theme:
        # this only works if there are only 2 histograms
        kwargs_a = {    'color' : 'tomato',
                    'filled'     : True,
                    #'edgecolor' : 'tomato',
                    #'hatch'     : '\\\\\\\\\\\\\\',
                    'alpha'     : 0.55,
                    'linewidth' : 2}
        kwargs_b = {    #'hatch' : '////////',\
                     'color'    :  'mediumblue',\
                     'filled'   : True,
                    #'edgecolor' : 'mediumblue',\
                    'linewidth' : 2,
                    'alpha'     : 0.65}
        kwargs = [kwargs_a, kwargs_b]
        #plt.hist(edges[:-1], bins=edges, weights=values1, histtype='step', hatch='////////', edgecolor='mediumblue', linewidth=2,density=True, alpha=0.65, label="McMurdo Data: Run 9125")
        #plt.hist(edges[:-1], bins=edges, weights=values2, histtype='stepfilled', color = 'tomato', hatch = '\\\\\\\\\\\\\\', edgecolor='tomato', linewidth=2, density=True, alpha=0.55, label="Muon MC")

        #plt.hist(edges[:-1], bins=edges, weights=values1, histtype='step', edgecolor='mediumblue', linewidth=2,density=True, alpha=0.75, label="McMurdo Data: Run 9125") 
    #kwargs.update({'label' : label})
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    ax = fig.gca()
    for idx,h_ in enumerate(h):
        h_.line(label=labels[idx],**kwargs[idx])
    #ax.legend(loc='upper right', frameon=False)
    ax.set_title(title, loc='right')
    ax.set_ylim(bottom=0)
    ax.set_xlabel(xlabel, loc='right')
    if log:    
        ax.set_yscale('symlog')
    ax.legend(frameon=False, loc='upper right')
    cb.visual.adjust_minor_ticks(ax, which='x')
    return fig

#################################################

def gander_2dplot(h      : d.histogram.hist2d,
                  xlabel : str,
                  ylabel : str,
                  title  : str,
                  xlim   = None,
                  invert_yaxis = False,
                  **kwargs) -> matplotlib.figure.Figure:
    """
    A plot with a default style for the 
    streamlit app

    # Arguemtns:

        **kwargs : to be passed to d.hist1d.line
    """
    if not kwargs:
        kwargs = {\
                  'cmap'   : matplotlib.colormaps['coolwarm'],
                  }
    #kwargs.update({'label' : label})
    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax = fig.gca()
    fig.set_facecolor("#0E1117")
    #ax.set_facecolor("#0E1117")
    cmap = kwargs['cmap'].copy()
    cmap.set_bad(color='#0E1117')
    #cmap.set_bad(color='red')
    kwargs['cmap'] = cmap
#285     bincontent, kwargs = _h2_transform_bins(self, kwargs)
#286             
#287     #masked_data = np.ma.masked_where(bincontent == 0, bincontent)
#288     #masked_data = np.
#289     masked_data = np.ma.masked_invalid(bincontent)
#290     kw = {"cmap": mpl.cm.jet, "aspect" : "auto", "interpolation" : "nearest" }
#291     kw.update(kwargs)
#292     print (bincontent)
#293     #print (masked_data)
#294     img = p.imshow(masked_data, origin="lower",
#295              extent=(self.binedges[0][0], self.binedges[0][-1], self.binedges[1][0], self.binedges[1][-1]),
#296              **kw)
#297     _h2label(self)
    #masked_data = np.ma.masked_invalid(h.bincontent)
    bincontent, __ = d.histviews._h2_transform_bins(h,kwargs)
    masked_data = np.ma.masked_where(bincontent == 0, bincontent)
    #masked_data = h.bincontent
    ax.imshow(masked_data, origin='lower',
              extent = (h.binedges[0][0], h.binedges[0][-1],
                        h.binedges[1][0], h.binedges[1][-1]),
              interpolation = 'nearest', aspect = 'auto',
              cmap = cmap)
    #h.imshow(**kwargs)
    #ax.legend(loc='upper right', frameon=False)
    ax.set_title(title, loc='right')
    ax.set_ylim(bottom=0)
    if xlim is not None:
        ax.set_xlim(left=xlim[0], right=xlim[1])
    ax.set_xlabel(xlabel, loc='right')
    ax.set_ylabel(ylabel, loc='top')
    cb.visual.adjust_minor_ticks(ax, which='both')
    ax.spines['top'].set_visible(True)
    ax.spines['right'].set_visible(True)
    if invert_yaxis:
        ax.invert_yaxis()
    return fig

